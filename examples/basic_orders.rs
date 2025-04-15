use orderly_connector_rs::{
    error::OrderlyError,
    rest::{client::Credentials, OrderlyService},
    types::{CreateOrderRequest, GetOrdersParams, OrderStatus, OrderType, Side},
};
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

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
    amount: Option<f64>,
) -> Result<u64, OrderlyError> {
    let order_req = CreateOrderRequest {
        symbol,
        order_type: OrderType::Market,
        side,
        order_price: None, // Market orders don't specify price
        order_quantity: Some(0.08),
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

/// Places a limit order
async fn place_limit_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    side: Side,
    quantity: f64,
    price: f64,
) -> Result<u64, OrderlyError> {
    info!(
        "Placing limit {:?} order for {} {} at {}",
        side, quantity, symbol, price
    );

    let order_req = CreateOrderRequest {
        symbol,
        order_type: OrderType::Limit,
        side,
        order_price: Some(price),
        order_quantity: Some(quantity),
        order_amount: None,
        client_order_id: None,
        visible_quantity: None,
    };

    match client.create_order(creds, order_req).await {
        Ok(resp) => {
            if resp.success {
                info!("Limit order placed successfully: ID {}", resp.data.order_id);
                Ok(resp.data.order_id)
            } else {
                Err(OrderlyError::ValidationError(
                    "Order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place limit order: {}", e);
            Err(e)
        }
    }
}

/// Cancels an order
async fn cancel_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    order_id: u64,
    symbol: &str,
) -> Result<(), OrderlyError> {
    info!("Cancelling order {} for {}", order_id, symbol);

    match client.cancel_order(creds, order_id, symbol).await {
        Ok(resp) => {
            if resp.success {
                info!("Order {} cancelled successfully", order_id);
                Ok(())
            } else {
                Err(OrderlyError::ValidationError(
                    "Order cancellation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to cancel order {}: {}", order_id, e);
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
            warn!("Order monitoring timed out after {} seconds", timeout_secs);
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

#[tokio::main]
async fn main() -> Result<(), OrderlyError> {
    // Setup logging
    setup_logging();
    info!("Starting basic orders example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)
        .map_err(|e| OrderlyError::ValidationError(format!("Failed to create client: {}", e)))?;

    let symbol = "PERP_SOL_USDC";
    info!("Using symbol: {}", symbol);

    // Example 1: Place and monitor a market buy order
    info!("Example 1: Market Buy Order");
    let collateral = 4.72;
    let leverage = 3.0;
    let amount = collateral * leverage;
    let market_order_id =
        place_market_order(&client, &creds, symbol, Side::Buy, Some(amount)).await?;
    let market_status = monitor_order(&client, &creds, market_order_id, 30).await?;
    info!("Market order final status: {}", market_status);
    // Example 1.5: Monitor order status
    let market_status = monitor_order(&client, &creds, market_order_id, 30).await?;
    info!("Market order final status: {}", market_status);
    if market_status == OrderStatus::Filled {
        info!("Market order filled, cancelling");
        // cancel_order(&client, &creds, market_order_id, symbol).await?;
    }
    // Example 2: Place and cancel a limit order
    info!("Example 2: Limit Sell Order");
    // Place limit order 5% above current price
    let limit_order_id =
        place_limit_order(&client, &creds, symbol, Side::Sell, 0.01, 2000.0).await?;

    // Wait a few seconds then cancel
    sleep(Duration::from_secs(5)).await;
    cancel_order(&client, &creds, limit_order_id, symbol).await?;

    // Example 3: Get all open orders
    info!("Example 3: List Open Orders");
    let params = GetOrdersParams {
        symbol: Some(symbol),
        status: Some(OrderStatus::New),
        ..Default::default()
    };

    match client.get_orders(&creds, Some(params)).await {
        Ok(resp) => {
            info!("Found {} open orders:", resp.data.rows.len());
            for order in resp.data.rows {
                info!(
                    "Order {}: {:?} {:?} {} @ {}",
                    order.order_id,
                    order.side,
                    order.order_type,
                    order.order_quantity.unwrap_or(0.0),
                    order.order_price.unwrap_or(0.0)
                );
            }
        }
        Err(e) => {
            error!("Failed to get open orders: {}", e);
        }
    }

    info!("Basic orders example completed");
    Ok(())
}
