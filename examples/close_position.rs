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

    // Convert environment variables to 'static strings
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

/// Places a market order
async fn place_market_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    side: Side,
    quantity: f64,
) -> Result<u64, OrderlyError> {
    info!(
        "Placing market {:?} order for {} {}",
        side, quantity, symbol
    );

    let order_req = CreateOrderRequest {
        symbol,
        order_type: OrderType::Market,
        side,
        order_price: None, // Market orders don't specify price
        order_quantity: Some(quantity),
        order_amount: None,
        client_order_id: None,
        visible_quantity: None,
    };

    match client.create_order(creds, order_req).await {
        Ok(resp) => {
            if resp.success {
                info!(
                    "Market order placed successfully: ID {}",
                    resp.data.order_id
                );
                Ok(resp.data.order_id)
            } else {
                Err(OrderlyError::ValidationError(
                    "Order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place market order: {}", e);
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

    Ok(OrderStatus::Expired) // Return expired if we timeout
}

/// Gets the current position for a symbol
async fn get_position(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
) -> Result<f64, OrderlyError> {
    match client.get_positions(creds).await {
        Ok(resp) => {
            // Find the position for our symbol
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

#[tokio::main]
async fn main() -> Result<(), OrderlyError> {
    // Setup logging
    setup_logging();
    info!("Starting position close example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)
        .map_err(|e| OrderlyError::ValidationError(format!("Failed to create client: {}", e)))?;

    let symbol = "PERP_SOL_USDC";
    info!("Using symbol: {}", symbol);

    // Step 1: Check current position
    let position_size = get_position(&client, &creds, symbol).await?;

    if position_size == 0.0 {
        // Open a new long position
        info!("Opening new long position");
        let quantity = 0.1; // Small test quantity
        let order_id = place_market_order(&client, &creds, symbol, Side::Buy, quantity).await?;
        let status = monitor_order(&client, &creds, order_id, 30).await?;

        if status != OrderStatus::Filled {
            return Err(OrderlyError::ValidationError(
                "Failed to open position".into(),
            ));
        }

        // Wait a moment and check the position
        sleep(Duration::from_secs(2)).await;
        let position_size = get_position(&client, &creds, symbol).await?;
        info!("Opened long position of size: {}", position_size);
    } else {
        info!("Found existing position of size: {}", position_size);
    }

    // Step 2: Close the position
    info!("Closing position");
    let close_side = if position_size > 0.0 {
        Side::Sell
    } else {
        Side::Buy
    };
    let close_quantity = position_size.abs();

    let close_order_id =
        place_market_order(&client, &creds, symbol, close_side, close_quantity).await?;
    let close_status = monitor_order(&client, &creds, close_order_id, 30).await?;

    if close_status == OrderStatus::Filled {
        info!("Position closed successfully");

        // Verify position is closed
        sleep(Duration::from_secs(2)).await;
        let final_position = get_position(&client, &creds, symbol).await?;
        info!("Final position size: {}", final_position);
    } else {
        error!("Failed to close position, final status: {}", close_status);
    }

    info!("Position close example completed");
    Ok(())
}
