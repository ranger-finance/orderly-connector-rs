use orderly_connector_rs::websocket::WebsocketPublicClient;
use serde_json::Value;
use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Get account ID from environment or use a default test account
    let account_id = env::var("ORDERLY_ACCOUNT_ID").unwrap_or_else(|_| {
        "0x60150d553f0ed15cf2c7fad91804d2548ee071a8450b0531bfb4f414823c69a8".to_string()
    });

    // Determine if testnet or mainnet
    let is_testnet = env::var("ORDERLY_TESTNET")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    println!("Starting Orderly Network Index Prices Monitor");
    println!("Account ID: {}", account_id);
    println!("Testnet: {}", is_testnet);

    // Create message handler to process incoming messages
    let message_handler = Arc::new(|msg: String| {
        // Parse the JSON message
        if let Ok(parsed) = serde_json::from_str::<Value>(&msg) {
            // Check if this is an index prices message
            if let Some(topic) = parsed.get("topic") {
                if topic == "indexprices" {
                    if let Some(data) = parsed.get("data") {
                        if let Some(prices) = data.as_array() {
                            println!("\n=== Index Prices Update ===");
                            println!("Timestamp: {}", parsed.get("ts").unwrap_or(&Value::Null));
                            for price in prices {
                                if let Some(symbol) = price.get("symbol") {
                                    if let Some(price_value) = price.get("price") {
                                        println!("{}: {}", symbol, price_value);
                                    }
                                }
                            }
                            println!("========================\n");
                        }
                    }
                }
            }
        }
    });

    // Create close handler
    let close_handler = Arc::new(|| {
        println!("Connection closed");
    });

    println!("Connecting to WebSocket...");

    // Connect to the WebSocket
    let client = match WebsocketPublicClient::connect(
        account_id.clone(),
        is_testnet,
        message_handler,
        close_handler,
    )
    .await
    {
        Ok(client) => {
            println!("Successfully connected to WebSocket");
            client
        }
        Err(e) => {
            eprintln!("Failed to connect to WebSocket: {}", e);
            return Err(e.into());
        }
    };

    // Wait a bit to ensure connection is established
    sleep(Duration::from_secs(2)).await;

    println!("Subscribing to index prices...");

    // Subscribe to index prices
    match client.subscribe_index_prices().await {
        Ok(_) => println!("Successfully subscribed to index prices. Waiting for updates..."),
        Err(e) => {
            eprintln!("Failed to subscribe to index prices: {}", e);
            return Err(e.into());
        }
    }

    // Keep the connection alive until Ctrl+C is pressed
    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            println!("Received Ctrl+C. Stopping...");
            client.stop().await;
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to wait for Ctrl+C: {}", e);
            Err(e.into())
        }
    }
}
