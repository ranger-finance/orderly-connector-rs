use orderly_connector_rs::{
    error::OrderlyError,
    rest::{client::Credentials, OrderlyService},
    types::{GetOrdersParams, OrderStatus},
};
use std::env;
use tokio::time::sleep;
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

/// Gets all pending orders for a symbol
async fn get_pending_orders(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: Option<&str>,
) -> Result<Vec<u64>, OrderlyError> {
    info!(
        "Getting pending orders{}",
        symbol.map_or("".into(), |s| format!(" for {}", s))
    );

    let params = GetOrdersParams {
        symbol,
        status: Some(OrderStatus::New), // Get only pending orders
        ..Default::default()
    };

    match client.get_orders(creds, Some(params)).await {
        Ok(resp) => {
            if resp.success {
                let order_ids: Vec<u64> =
                    resp.data.rows.iter().map(|order| order.order_id).collect();
                info!("Found {} pending orders", order_ids.len());
                Ok(order_ids)
            } else {
                Err(OrderlyError::ValidationError("Failed to get orders".into()))
            }
        }
        Err(e) => {
            error!("Failed to get pending orders: {}", e);
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

#[tokio::main]
async fn main() -> Result<(), OrderlyError> {
    // Setup logging
    setup_logging();
    info!("Starting cancel orders example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)
        .map_err(|e| OrderlyError::ValidationError(format!("Failed to create client: {}", e)))?;

    // Example 1: Cancel all pending orders for a specific symbol
    let symbol = "PERP_SOL_USDC";
    info!("Example 1: Cancelling all pending orders for {}", symbol);

    let pending_orders = get_pending_orders(&client, &creds, Some(symbol)).await?;

    for order_id in pending_orders {
        match cancel_order(&client, &creds, order_id, symbol).await {
            Ok(_) => info!("Successfully cancelled order {}", order_id),
            Err(e) => error!("Failed to cancel order {}: {}", order_id, e),
        }
        // Add a small delay between cancellations
        sleep(std::time::Duration::from_millis(100)).await;
    }

    // Example 2: Cancel all pending orders across all symbols
    info!("Example 2: Cancelling all pending orders across all symbols");

    let all_pending_orders = get_pending_orders(&client, &creds, None).await?;

    // For each order, we need to get its symbol first
    for order_id in all_pending_orders {
        match client.get_order(&creds, order_id).await {
            Ok(order_details) => {
                let symbol = &order_details.data.order.symbol;
                match cancel_order(&client, &creds, order_id, symbol).await {
                    Ok(_) => info!("Successfully cancelled order {} for {}", order_id, symbol),
                    Err(e) => error!("Failed to cancel order {} for {}: {}", order_id, symbol, e),
                }
            }
            Err(e) => error!("Failed to get details for order {}: {}", order_id, e),
        }
        // Add a small delay between cancellations
        sleep(std::time::Duration::from_millis(100)).await;
    }

    info!("Cancel orders example completed");
    Ok(())
}
