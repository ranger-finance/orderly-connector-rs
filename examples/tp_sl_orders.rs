use orderly_connector_rs::{
    error::OrderlyError,
    rest::{client::Credentials, OrderlyService},
    types::{CreateOrderRequest, OrderStatus, OrderType, Side},
};
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

/// Sets up logging with a custom format
fn setup_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

/// Loads environment variables and creates credentials
fn load_credentials() -> Result<(Credentials<'static>, bool), OrderlyError> {
    dotenv::dotenv().ok();

    let api_key = env::var("ORDERLY_API_KEY")
        .map_err(|_| OrderlyError::ValidationError("ORDERLY_API_KEY not set".into()))?;
    let secret = env::var("ORDERLY_SECRET")
        .map_err(|_| OrderlyError::ValidationError("ORDERLY_SECRET not set".into()))?;
    let account_id = env::var("ORDERLY_ACCOUNT_ID")
        .map_err(|_| OrderlyError::ValidationError("ORDERLY_ACCOUNT_ID not set".into()))?;

    let api_key = Box::leak(api_key.into_boxed_str());
    let secret = Box::leak(secret.into_boxed_str());
    let account_id = Box::leak(account_id.into_boxed_str());

    let is_testnet: bool = env::var("ORDERLY_TESTNET")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .map_err(|_| OrderlyError::ValidationError("Invalid ORDERLY_TESTNET value".into()))?;

    let creds = Credentials {
        orderly_key: api_key,
        orderly_secret: secret,
        orderly_account_id: account_id,
    };

    Ok((creds, is_testnet))
}

/// Places a take profit limit order
async fn place_take_profit_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    quantity: f64,
    price: f64,
    side: Side,
) -> Result<u64, OrderlyError> {
    info!(
        "Placing take profit {:?} order for {} {} at {}",
        side, quantity, symbol, price
    );

    let order_req = CreateOrderRequest {
        symbol,
        order_type: OrderType::Limit,
        side,
        order_price: Some(price),
        order_quantity: Some(quantity),
        order_amount: None,
        client_order_id: Some("tp_order"),
        visible_quantity: None,
    };

    match client.create_order(creds, order_req).await {
        Ok(resp) => {
            if resp.success {
                info!(
                    "Take profit order placed successfully: ID {}",
                    resp.data.order_id
                );
                Ok(resp.data.order_id)
            } else {
                Err(OrderlyError::ValidationError(
                    "Take profit order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place take profit order: {}", e);
            Err(e)
        }
    }
}

/// Places a stop loss market order
async fn place_stop_loss_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    quantity: f64,
    trigger_price: f64,
    side: Side,
) -> Result<u64, OrderlyError> {
    info!(
        "Placing stop loss {:?} order for {} {} at trigger price {}",
        side, quantity, symbol, trigger_price
    );

    let order_req = CreateOrderRequest {
        symbol,
        order_type: OrderType::Market,
        side,
        order_price: None,
        order_quantity: Some(quantity),
        order_amount: None,
        client_order_id: Some("sl_order"),
        visible_quantity: None,
    };

    match client.create_order(creds, order_req).await {
        Ok(resp) => {
            if resp.success {
                info!(
                    "Stop loss order placed successfully: ID {}",
                    resp.data.order_id
                );
                Ok(resp.data.order_id)
            } else {
                Err(OrderlyError::ValidationError(
                    "Stop loss order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place stop loss order: {}", e);
            Err(e)
        }
    }
}

/// Gets the current position for a symbol
async fn get_position(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
) -> Result<f64, OrderlyError> {
    match client.get_positions(creds).await {
        Ok(resp) => {
            if let Some(position) = resp.data.rows.iter().find(|p| p.symbol == symbol) {
                info!(
                    "Current position for {}: {} (PnL: {})",
                    symbol, position.position_qty, position.unsettled_pnl
                );
                Ok(position.position_qty)
            } else {
                info!("No position found for {}", symbol);
                Ok(0.0)
            }
        }
        Err(e) => {
            error!("Failed to get positions: {}", e);
            Err(e)
        }
    }
}

/// Monitors order status until filled or cancelled
async fn monitor_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    order_id: u64,
    timeout_secs: u64,
) -> Result<OrderStatus, OrderlyError> {
    info!("Monitoring order {} for {} seconds", order_id, timeout_secs);
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            error!("Order monitoring timed out after {} seconds", timeout_secs);
            break;
        }

        match client.get_order(creds, order_id).await {
            Ok(resp) => {
                let status = resp.data.order.status;
                info!("Order {} status: {}", order_id, status);

                match status {
                    OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected => {
                        return Ok(status)
                    }
                    OrderStatus::PartialFilled => {
                        info!(
                            "Order {} partially filled: {} / {}",
                            order_id,
                            resp.data.order.executed_quantity.unwrap_or(0.0),
                            resp.data.order.order_quantity.unwrap_or(0.0)
                        );
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("Error checking order status: {}", e);
                return Err(e);
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    Ok(OrderStatus::Expired)
}

#[tokio::main]
async fn main() -> Result<(), OrderlyError> {
    // Setup logging
    setup_logging();
    info!("Starting TP/SL orders example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)
        .map_err(|e| OrderlyError::ValidationError(format!("Failed to create client: {}", e)))?;

    let symbol = "PERP_ETH_USDC";
    info!("Using symbol: {}", symbol);

    // Step 1: Check current position
    let position_size = get_position(&client, &creds, symbol).await?;

    if position_size == 0.0 {
        // Example values - adjust these based on your strategy
        let entry_price = 2000.0;
        let quantity = 0.1;
        let take_profit_price = entry_price * 1.05; // 5% profit target
        let stop_loss_price = entry_price * 0.95; // 5% loss limit

        // Step 2: Place entry order (market order for immediate execution)
        info!("Opening new long position with market order");
        let entry_order = CreateOrderRequest {
            symbol,
            order_type: OrderType::Market,
            side: Side::Buy,
            order_price: None,
            order_quantity: Some(quantity),
            order_amount: None,
            client_order_id: Some("entry_long"),
            visible_quantity: None,
        };

        let entry_order_id = client
            .create_order(&creds, entry_order)
            .await?
            .data
            .order_id;

        // Wait for entry order to fill
        match monitor_order(&client, &creds, entry_order_id, 30).await? {
            OrderStatus::Filled => {
                info!("Entry order filled successfully");

                // Step 3: Place take profit order
                let tp_order_id = place_take_profit_order(
                    &client,
                    &creds,
                    symbol,
                    quantity,
                    take_profit_price,
                    Side::Sell,
                )
                .await?;

                // Step 4: Place stop loss order
                let sl_order_id = place_stop_loss_order(
                    &client,
                    &creds,
                    symbol,
                    quantity,
                    stop_loss_price,
                    Side::Sell,
                )
                .await?;

                info!("TP order ID: {}, SL order ID: {}", tp_order_id, sl_order_id);

                // Monitor both orders
                loop {
                    let tp_status = monitor_order(&client, &creds, tp_order_id, 10).await?;
                    let sl_status = monitor_order(&client, &creds, sl_order_id, 10).await?;

                    match (tp_status, sl_status) {
                        (OrderStatus::Filled, _) => {
                            info!("Take profit order filled!");
                            break;
                        }
                        (_, OrderStatus::Filled) => {
                            info!("Stop loss triggered!");
                            break;
                        }
                        _ => {
                            info!("Both orders still active, continuing to monitor...");
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
            _ => {
                error!("Entry order did not fill");
                return Err(OrderlyError::ValidationError("Entry order failed".into()));
            }
        }
    } else {
        info!("Found existing position of size: {}", position_size);
    }

    info!("TP/SL orders example completed");
    Ok(())
}
