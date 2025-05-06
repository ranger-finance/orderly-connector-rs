use orderly_connector_rs::{
    error::OrderlyError,
    rest::{client::Credentials, OrderlyService},
    types::{AlgoOrderType, CreateAlgoOrderRequest, Side},
};
use std::env;
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

/// Places a stop-loss order
async fn place_stop_loss(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    quantity: f64,
    trigger_price: f64,
) -> Result<String, OrderlyError> {
    info!(
        "Placing stop-loss order for {} {} at trigger price {}",
        quantity, symbol, trigger_price
    );

    let request = CreateAlgoOrderRequest {
        symbol: symbol.to_string(),
        order_type: AlgoOrderType::StopMarket,
        side: Side::Sell,
        quantity,
        trigger_price,
        limit_price: None,
        trailing_delta: None,
        client_order_id: Some(format!(
            "stop_loss_{}",
            chrono::Utc::now().timestamp_millis()
        )),
        reduce_only: Some(true),
    };

    match client.create_algo_order(creds, request).await {
        Ok(resp) => {
            if resp.success {
                info!(
                    "Stop-loss order placed successfully: ID {}",
                    resp.data.algo_order_id
                );
                Ok(resp.data.algo_order_id.to_string())
            } else {
                Err(OrderlyError::ValidationError(
                    "Order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place stop-loss order: {}", e);
            Err(e)
        }
    }
}

/// Places a take-profit order
async fn place_take_profit(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    quantity: f64,
    trigger_price: f64,
    limit_price: f64,
) -> Result<String, OrderlyError> {
    info!(
        "Placing take-profit order for {} {} at trigger price {} with limit {}",
        quantity, symbol, trigger_price, limit_price
    );

    let request = CreateAlgoOrderRequest {
        symbol: symbol.to_string(),
        order_type: AlgoOrderType::TakeProfitLimit,
        side: Side::Sell,
        quantity,
        trigger_price,
        limit_price: Some(limit_price),
        trailing_delta: None,
        client_order_id: Some(format!(
            "take_profit_{}",
            chrono::Utc::now().timestamp_millis()
        )),
        reduce_only: Some(true),
    };

    match client.create_algo_order(creds, request).await {
        Ok(resp) => {
            if resp.success {
                info!(
                    "Take-profit order placed successfully: ID {}",
                    resp.data.algo_order_id
                );
                Ok(resp.data.algo_order_id.to_string())
            } else {
                Err(OrderlyError::ValidationError(
                    "Order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place take-profit order: {}", e);
            Err(e)
        }
    }
}

/// Places a trailing stop order
async fn place_trailing_stop(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    quantity: f64,
    trailing_delta: f64,
) -> Result<String, OrderlyError> {
    info!(
        "Placing trailing stop order for {} {} with delta {}",
        quantity, symbol, trailing_delta
    );

    let request = CreateAlgoOrderRequest {
        symbol: symbol.to_string(),
        order_type: AlgoOrderType::TrailingStop,
        side: Side::Sell,
        quantity,
        trigger_price: 0.0, // Not used for trailing stops
        limit_price: None,
        trailing_delta: Some(trailing_delta),
        client_order_id: Some(format!(
            "trailing_stop_{}",
            chrono::Utc::now().timestamp_millis()
        )),
        reduce_only: Some(true),
    };

    match client.create_algo_order(creds, request).await {
        Ok(resp) => {
            if resp.success {
                info!(
                    "Trailing stop order placed successfully: ID {}",
                    resp.data.algo_order_id
                );
                Ok(resp.data.algo_order_id.to_string())
            } else {
                Err(OrderlyError::ValidationError(
                    "Order creation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to place trailing stop order: {}", e);
            Err(e)
        }
    }
}

/// Cancels an algorithmic order
async fn cancel_algo_order(
    client: &OrderlyService,
    creds: &Credentials<'_>,
    symbol: &str,
    algo_order_id: &str,
) -> Result<(), OrderlyError> {
    info!("Cancelling algo order {} for {}", algo_order_id, symbol);

    match client.cancel_algo_order(creds, symbol, algo_order_id).await {
        Ok(resp) => {
            if resp.success {
                info!("Algo order {} cancelled successfully", algo_order_id);
                Ok(())
            } else {
                Err(OrderlyError::ValidationError(
                    "Order cancellation failed".into(),
                ))
            }
        }
        Err(e) => {
            error!("Failed to cancel algo order {}: {}", algo_order_id, e);
            Err(e)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), OrderlyError> {
    // Initialize logging
    setup_logging();
    info!("Starting advanced orders example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)
        .map_err(|e| OrderlyError::ValidationError(format!("Failed to create client: {}", e)))?;

    let symbol = "PERP_ETH_USDC";
    let quantity = 0.1;

    // Example 1: Place a stop-loss order
    info!("Example 1: Stop-Loss Order");
    let stop_loss_id = place_stop_loss(&client, &creds, symbol, quantity, 1800.0).await?;
    info!("Created stop-loss order: {}", stop_loss_id);

    // Example 2: Place a take-profit order
    info!("Example 2: Take-Profit Order");
    let take_profit_id =
        place_take_profit(&client, &creds, symbol, quantity, 2200.0, 2190.0).await?;
    info!("Created take-profit order: {}", take_profit_id);

    // Example 3: Place a trailing stop order
    info!("Example 3: Trailing Stop Order");
    let trailing_stop_id = place_trailing_stop(&client, &creds, symbol, quantity, 50.0).await?;
    info!("Created trailing stop order: {}", trailing_stop_id);

    // Wait a few seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Cancel all algo orders
    info!("Cancelling all algo orders");
    for order_id in [&stop_loss_id, &take_profit_id, &trailing_stop_id] {
        if let Err(e) = cancel_algo_order(&client, &creds, symbol, order_id).await {
            error!("Failed to cancel order {}: {}", order_id, e);
        }
    }

    info!("Advanced orders example completed");
    Ok(())
}
