use orderly_connector_rs::{
    error::OrderlyError,
    rest::{client::Credentials, OrderlyService},
    types::{CreateOrderRequest, OrderStatus, OrderType, Side},
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
        symbol: symbol.to_string(),
        order_type: OrderType::Limit,
        side,
        order_price: Some(price),
        order_quantity: Some(quantity),
        order_amount: None,
        client_order_id: Some("my_order_id".to_string()),
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
    info!("Starting limit orders example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)
        .map_err(|e| OrderlyError::ValidationError(format!("Failed to create client: {}", e)))?;

    let symbol = "PERP_SOL_USDC";
    info!("Using symbol: {}", symbol);

    // Step 1: Check current position
    info!("Step 1: Checking current position for {}", symbol);
    let position_size = get_position(&client, &creds, symbol).await?;
    info!("Current position size: {}", position_size);

    if position_size == 0.0 {
        // Step 2: Open a new long position with a limit order
        info!("Step 2: Opening new long position with limit order");
        let quantity = 0.1; // Small test quantity
        let entry_price = 100.0; // Set your desired entry price
        info!(
            "Attempting to open long position: quantity={}, price={}",
            quantity, entry_price
        );

        let order_id =
            place_limit_order(&client, &creds, symbol, Side::Buy, quantity, entry_price).await?;
        info!("Limit buy order placed with ID: {}", order_id);

        // Monitor the order with a timeout
        info!("Monitoring order {} for execution...", order_id);
        let status = monitor_order(&client, &creds, order_id, 30).await?;
        info!("Order {} final status: {}", order_id, status);

        match status {
            OrderStatus::Filled => {
                info!("Order {} was filled, checking position...", order_id);
                // Wait a moment and check the position
                sleep(Duration::from_secs(2)).await;
                let position_size = get_position(&client, &creds, symbol).await?;
                info!("Updated position size after fill: {}", position_size);
            }
            OrderStatus::Cancelled | OrderStatus::Rejected => {
                info!("Order {} was {} - aborting", order_id, status);
                return Err(OrderlyError::ValidationError(
                    format!("Failed to open position - order was {}", status).into(),
                ));
            }
            _ => {
                // For any other status (including timeout/expired), cancel the order and inform
                info!(
                    "Order {} not filled within timeout (status: {}), cancelling",
                    order_id, status
                );
                match cancel_order(&client, &creds, order_id, symbol).await {
                    Ok(_) => info!("Successfully cancelled unfilled order {}", order_id),
                    Err(e) => error!("Failed to cancel order {}: {}", order_id, e),
                }
                return Err(OrderlyError::ValidationError(
                    format!("Failed to open position - order {} timed out", order_id).into(),
                ));
            }
        }
    } else {
        info!("Found existing position of size: {}", position_size);
    }

    // Step 3: Place a limit order to close the position
    info!(
        "Step 3: Placing limit order to close position of size {}",
        position_size
    );
    let close_side = if position_size > 0.0 {
        Side::Sell
    } else {
        Side::Buy
    };
    let close_quantity = position_size.abs();
    let close_price = if position_size > 0.0 { 65.0 } else { 55.0 }; // Set take-profit price
    info!(
        "Closing position with {:?} order: quantity={}, price={}",
        close_side, close_quantity, close_price
    );

    let close_order_id = place_limit_order(
        &client,
        &creds,
        symbol,
        close_side,
        close_quantity,
        close_price,
    )
    .await?;
    info!("Close order placed with ID: {}", close_order_id);

    // Monitor the closing order
    info!("Monitoring close order {} for execution...", close_order_id);
    let close_status = monitor_order(&client, &creds, close_order_id, 30).await?;
    info!(
        "Close order {} final status: {}",
        close_order_id, close_status
    );

    match close_status {
        OrderStatus::Filled => {
            info!(
                "Close order {} was filled, verifying position closure...",
                close_order_id
            );
            // Verify position is closed
            sleep(Duration::from_secs(2)).await;
            let final_position = get_position(&client, &creds, symbol).await?;
            info!(
                "Final position size after close: {} (expected: 0.0)",
                final_position
            );
            if final_position != 0.0 {
                warn!(
                    "Position not fully closed! Remaining size: {}",
                    final_position
                );
            }
        }
        OrderStatus::Cancelled | OrderStatus::Rejected => {
            info!(
                "Close order {} was {}, attempting cancellation if needed",
                close_order_id, close_status
            );
            if close_status != OrderStatus::Cancelled {
                match cancel_order(&client, &creds, close_order_id, symbol).await {
                    Ok(_) => info!("Successfully cancelled close order {}", close_order_id),
                    Err(e) => error!("Failed to cancel close order {}: {}", close_order_id, e),
                }
            }
        }
        _ => {
            // Order timed out, cancel it
            info!(
                "Close order {} timed out (status: {}), cancelling",
                close_order_id, close_status
            );
            match cancel_order(&client, &creds, close_order_id, symbol).await {
                Ok(_) => info!(
                    "Successfully cancelled timed out close order {}",
                    close_order_id
                ),
                Err(e) => error!(
                    "Failed to cancel timed out close order {}: {}",
                    close_order_id, e
                ),
            }
        }
    }

    info!("Limit orders example completed");
    Ok(())
}
