// examples/ws_private.rs
use orderly_connector_rs::websocket::WebsocketPrivateClient;
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
    let api_key = env::var("ORDERLY_API_KEY").expect("ORDERLY_API_KEY not set");
    let api_secret = env::var("ORDERLY_SECRET").expect("ORDERLY_SECRET not set");
    let account_id = env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID not set");
    let is_testnet: bool = env::var("ORDERLY_TESTNET")
        .unwrap_or("true".to_string())
        .parse()
        .expect("ORDERLY_TESTNET must be true or false");

    // Ensure we run this example against testnet only
    if !is_testnet {
        eprintln!("This example should only be run against the testnet.");
        eprintln!("Set ORDERLY_TESTNET=true in your environment variables or .env file.");
        return;
    }

    println!(
        "Connecting to Private WebSocket (Testnet: {})...",
        is_testnet
    );

    // Connect the client
    let client = match WebsocketPrivateClient::connect(
        api_key,
        api_secret,
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

    println!("Connected. Subscribing to private topics...");

    // Subscribe to execution reports
    if let Err(e) = client.subscribe_execution_reports().await {
        eprintln!("Error subscribing to execution reports: {}", e);
    }

    // Subscribe to balance updates
    if let Err(e) = client.subscribe_balance().await {
        eprintln!("Error subscribing to balance updates: {}", e);
    }

    // Keep the connection alive for a while
    println!("Listening for messages for 30 seconds...");
    println!("Try placing/cancelling an order via the REST API example or UI to see messages.");
    sleep(Duration::from_secs(30)).await;

    // Unsubscribe
    println!("Unsubscribing...");
    if let Err(e) = client.unsubscribe_execution_reports().await {
        eprintln!("Error unsubscribing from execution reports: {}", e);
    }
    if let Err(e) = client.unsubscribe_balance().await {
        eprintln!("Error unsubscribing from balance: {}", e);
    }

    // Give time for unsubscribe confirmation
    sleep(Duration::from_secs(2)).await;

    // Stop the client
    println!("Stopping client...");
    client.stop().await;

    println!("Example finished.");
}
