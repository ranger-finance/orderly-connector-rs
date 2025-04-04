// examples/ws_public.rs
use orderly_connector_rs::websocket::WebsocketPublicClient;
use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

// Simple message handler that just prints the message
fn message_handler(message: String) {
    println!("Received Message: {}", message);
}

// Simple close handler
fn close_handler() {
    println!("Connection Closed.");
}

#[tokio::main]
async fn main() {
    // Initialize logging (optional)
    env_logger::init();

    // Optional: Load .env file if you have one
    dotenv::dotenv().ok();

    // Load configuration from environment variables
    // Public WS needs account ID for the URL path
    let account_id = env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID not set");
    let is_testnet: bool = env::var("ORDERLY_TESTNET")
        .unwrap_or("true".to_string())
        .parse()
        .expect("ORDERLY_TESTNET must be true or false");

    println!(
        "Connecting to Public WebSocket (Testnet: {})...",
        is_testnet
    );

    // Connect the client
    let client = match WebsocketPublicClient::connect(
        account_id,
        is_testnet,
        Arc::new(message_handler),
        Arc::new(close_handler),
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    println!("Connected. Subscribing to topics...");

    // Subscribe to tickers
    if let Err(e) = client.subscribe_tickers().await {
        eprintln!("Error subscribing to tickers: {}", e);
    }

    // Subscribe to orderbook for a symbol
    let symbol = "PERP_ETH_USDC";
    if let Err(e) = client.subscribe_orderbook(symbol).await {
        eprintln!("Error subscribing to orderbook for {}: {}", symbol, e);
    }

    // Keep the connection alive for a while
    println!("Listening for messages for 30 seconds...");
    sleep(Duration::from_secs(30)).await;

    // Unsubscribe
    println!("Unsubscribing...");
    if let Err(e) = client.unsubscribe_orderbook(symbol).await {
        eprintln!("Error unsubscribing from orderbook: {}", e);
    }
    if let Err(e) = client.unsubscribe_tickers().await {
        eprintln!("Error unsubscribing from tickers: {}", e);
    }

    // Give time for unsubscribe confirmation
    sleep(Duration::from_secs(2)).await;

    // Stop the client
    println!("Stopping client...");
    client.stop().await;

    println!("Example finished.");
}
