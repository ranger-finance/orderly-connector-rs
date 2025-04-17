use orderly_connector_rs::{
    rest::{client::Credentials, OrderlyService},
    types::{CreateOrderRequest, OrderType, Side},
};
use std::env;
use tokio::time::{sleep, Duration};
use tracing::info;

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
fn load_credentials() -> Result<(Credentials<'static>, bool), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let api_key = env::var("ORDERLY_API_KEY")?;
    let secret = env::var("ORDERLY_SECRET")?;
    let account_id = env::var("ORDERLY_ACCOUNT_ID")?;

    // Convert environment variables to 'static strings
    let api_key = Box::leak(api_key.into_boxed_str());
    let secret = Box::leak(secret.into_boxed_str());
    let account_id = Box::leak(account_id.into_boxed_str());

    let is_testnet: bool = env::var("ORDERLY_TESTNET")
        .unwrap_or_else(|_| "true".to_string())
        .parse()?;

    let creds = Credentials {
        orderly_key: api_key,
        orderly_secret: secret,
        orderly_account_id: account_id,
    };

    Ok((creds, is_testnet))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    setup_logging();
    info!("Starting cancel order example");

    // Load credentials and create client
    let (creds, is_testnet) = load_credentials()?;
    info!("Using testnet: {}", is_testnet);

    let client = OrderlyService::new(is_testnet, None)?;

    // Get account information before placing order
    info!("Fetching account information...");
    let account_info = client.get_account_info(&creds).await?;
    info!("Account Info: {:?}", account_info);

    // Get current positions
    info!("Fetching positions...");
    let positions = client.get_positions(&creds).await?;
    info!("Current Positions: {:?}", positions);

    // Create a limit order
    let symbol = "PERP_SOL_USDC";
    let order_request = CreateOrderRequest {
        symbol: symbol.to_string(),
        order_type: OrderType::Limit,
        side: Side::Buy,
        order_quantity: Some(0.1),
        order_price: Some(100.0),
        order_amount: None,
        visible_quantity: None,
        client_order_id: Some("my_order_id".to_string()),
    };

    // Place the order
    info!("Placing limit order...");
    let order_response = client.create_order(&creds, order_request).await?;
    info!("Order placed: {:?}", order_response);

    // Wait a bit before cancelling
    sleep(Duration::from_secs(2)).await;

    // Cancel the order
    info!("Cancelling order...");
    let cancel_response = client
        .cancel_order(&creds, order_response.data.order_id, symbol)
        .await?;
    info!("Cancel response: {:?}", cancel_response);

    Ok(())
}
