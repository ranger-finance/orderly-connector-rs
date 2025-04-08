use orderly_connector_rs::websocket::WebsocketPublicClient;
use serde_json::json;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for better visibility
    env_logger::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get account ID from environment (or use your hard-coded one for testing)
    let account_id = "0x60150d553f0ed15cf2c7fad91804d2548ee071a8450b0531bfb4f414823c69a8";

    // Using mainnet for production use
    let is_testnet: bool = false;

    // Define WebSocket URLs as per EVM documentation
    let ws_base_url = if is_testnet {
        "wss://testnet-ws-evm.orderly.org/ws/stream"
    } else {
        "wss://ws-evm.orderly.org/ws/stream"
    };

    // Create a logfile for the WebSocket traffic
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let log_filename = format!("websocket_log_{}.txt", timestamp);
    println!("Logging WebSocket traffic to: {}", log_filename);

    // Initialize log file
    {
        let mut file = File::create(&log_filename)?;
        writeln!(
            file,
            "WebSocket Session Log - Started at: {}",
            chrono::Local::now()
        )?;
        writeln!(file, "")?;
    }

    println!("Using account ID: {}", account_id);
    println!(
        "Connecting to {} WebSocket: {}/{}",
        if is_testnet { "Testnet" } else { "Mainnet" },
        ws_base_url,
        account_id
    );

    // Optional WebSocket ID for message tracking
    let wss_id = env::var("ORDERLY_WSS_ID").unwrap_or_else(|_| "liquidation_stream".to_string());
    println!("Using WebSocket ID: {}", wss_id);

    // Log connection attempt
    {
        let mut file = OpenOptions::new().append(true).open(&log_filename)?;
        writeln!(
            file,
            "[{}] Attempting to connect to {}/{}",
            chrono::Local::now(),
            ws_base_url,
            account_id
        )?;
    }

    // Create message handler that logs and prints all received messages
    let log_filename_clone = log_filename.clone();
    let message_handler = Arc::new(move |msg: String| {
        let timestamp = chrono::Local::now();
        println!("Received message: {}", msg);

        // Append to log file
        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename_clone) {
            if let Err(e) = writeln!(file, "[{}] RECEIVED: {}", timestamp, msg) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }

        // Log if we see ping messages
        if msg.contains("ping") {
            println!("Ping received - client should handle automatically");
            if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename_clone) {
                let _ = writeln!(file, "[{}] Ping detected, handled by client", timestamp);
            }
        }
    });

    // Create close handler
    let log_filename_clone = log_filename.clone();
    let close_handler = Arc::new(move || {
        let timestamp = chrono::Local::now();
        println!("WebSocket connection closed");

        // Append to log file
        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename_clone) {
            if let Err(e) = writeln!(file, "[{}] CONNECTION CLOSED", timestamp) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    });

    // Connect to the WebSocket server
    let client = match WebsocketPublicClient::connect(
        account_id.to_string(),
        is_testnet,
        message_handler.clone(),
        close_handler.clone(),
    )
    .await
    {
        Ok(client) => {
            // Log successful connection
            if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
                let _ = writeln!(
                    file,
                    "[{}] Connected successfully to {}/{}",
                    chrono::Local::now(),
                    ws_base_url,
                    account_id
                );
            }

            println!("Connected successfully to {}/{}", ws_base_url, account_id);
            client
        }
        Err(e) => {
            // Log connection error
            if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
                let _ = writeln!(file, "[{}] Connection failed: {}", chrono::Local::now(), e);
            }

            eprintln!("Failed to connect: {}", e);
            return Err(e.into());
        }
    };

    // IMPORTANT: Wait a moment after connecting before subscribing
    println!("Waiting for connection to stabilize...");
    if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
        let _ = writeln!(
            file,
            "[{}] Waiting 3 seconds for connection to stabilize",
            chrono::Local::now()
        );
    }
    sleep(Duration::from_secs(3)).await;

    println!("Subscribing to liquidations stream...");

    // Log subscription attempt
    if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
        let _ = writeln!(
            file,
            "[{}] Attempting to subscribe to liquidations stream",
            chrono::Local::now()
        );
    }

    // Create an explicit subscription message following the EVM API format
    // as seen in the Python connector implementation
    let subscription_message = json!({
        "id": wss_id,
        "event": "subscribe",
        "topic": "liquidation"
    });

    if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
        let _ = writeln!(
            file,
            "[{}] Created subscription message: {}",
            chrono::Local::now(),
            subscription_message
        );
    }

    // Try using the standard subscribe_liquidations method
    println!("Trying standard subscribe_liquidations method...");
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;

    while retry_count < MAX_RETRIES {
        match client.subscribe_liquidations().await {
            Ok(_) => {
                if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
                    let _ = writeln!(
                        file,
                        "[{}] Successfully subscribed to liquidations stream",
                        chrono::Local::now()
                    );
                }
                println!("Successfully subscribed to liquidations stream");
                break;
            }
            Err(e) => {
                retry_count += 1;
                if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
                    let _ = writeln!(
                        file,
                        "[{}] Attempt {}/{}: Failed to subscribe to liquidations: {}",
                        chrono::Local::now(),
                        retry_count,
                        MAX_RETRIES,
                        e
                    );
                }
                eprintln!(
                    "Attempt {}/{}: Failed to subscribe to liquidations: {}",
                    retry_count, MAX_RETRIES, e
                );

                if retry_count == MAX_RETRIES {
                    if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
                        let _ = writeln!(
                            file,
                            "[{}] Maximum retries reached. Will still wait for messages...",
                            chrono::Local::now()
                        );
                    }
                    // Don't exit on failure, just continue to listen
                    println!("Subscription failed, but continuing to listen for any messages...");
                }
                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    println!("Waiting for liquidation events... Press Ctrl+C to exit");

    // Log waiting status
    if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
        let _ = writeln!(
            file,
            "[{}] Waiting for liquidation events...",
            chrono::Local::now()
        );
    }

    // Keep the connection alive with heartbeat logging
    loop {
        sleep(Duration::from_secs(30)).await;
        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_filename) {
            let _ = writeln!(
                file,
                "[{}] Heartbeat - Connection still alive",
                chrono::Local::now()
            );
        }
    }
}
