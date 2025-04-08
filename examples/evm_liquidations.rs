use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

// Simple function to log to a file with timestamp
async fn log_to_file(filename: &str, message: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().append(true).open(filename)?;
    writeln!(file, "[{}] {}", chrono::Local::now(), message)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for better visibility
    env_logger::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get account ID from environment or use provided one
    let account_id = env::var("ORDERLY_ACCOUNT_ID").unwrap_or_else(|_| {
        "0x60150d553f0ed15cf2c7fad91804d2548ee071a8450b0531bfb4f414823c69a8".to_string()
    });

    // Determine if testnet or mainnet
    let is_testnet = env::var("ORDERLY_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    // Define WebSocket URL as per Orderly EVM documentation
    let ws_base_url = if is_testnet {
        "wss://testnet-ws-evm.orderly.org/ws/stream"
    } else {
        "wss://ws-evm.orderly.org/ws/stream"
    };

    let ws_url = format!("{}/{}", ws_base_url, account_id);

    // Create log file
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let log_filename = format!("evm_websocket_log_{}.txt", timestamp);

    // Initialize log file
    {
        let mut file = File::create(&log_filename)?;
        writeln!(
            file,
            "EVM WebSocket Session Log - Started at: {}",
            chrono::Local::now()
        )?;
        writeln!(file, "URL: {}", ws_url)?;
        writeln!(file, "Account ID: {}", account_id)?;
        writeln!(file, "Testnet: {}", is_testnet)?;
        writeln!(file, "")?;
    }

    println!("Connecting to Orderly EVM WebSocket at: {}", ws_url);
    println!("Logging to {}", log_filename);

    // Parse URL
    let url = Url::parse(&ws_url)?;

    // Connect to WebSocket server
    log_to_file(&log_filename, &format!("Connecting to {}", ws_url)).await?;

    let (ws_stream, _) = match connect_async(url).await {
        Ok(conn) => {
            println!("Connected to WebSocket server");
            log_to_file(&log_filename, "Connected successfully").await?;
            conn
        }
        Err(e) => {
            let err_msg = format!("Failed to connect: {}", e);
            println!("{}", err_msg);
            log_to_file(&log_filename, &err_msg).await?;
            return Err(e.into());
        }
    };

    println!("WebSocket handshake completed");

    // Create a channel to send messages to the WebSocket
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    // Split the WebSocket
    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to handle outgoing messages
    let write_task = {
        let log_filename = log_filename.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Message::Text(ref text) = msg {
                    if let Err(e) = log_to_file(&log_filename, &format!("SENDING: {}", text)).await
                    {
                        eprintln!("Failed to log outgoing message: {}", e);
                    }
                }

                if let Err(e) = write.send(msg).await {
                    eprintln!("Failed to send message: {}", e);
                    if let Err(e) =
                        log_to_file(&log_filename, &format!("Error sending message: {}", e)).await
                    {
                        eprintln!("Failed to log error: {}", e);
                    }
                    break;
                }
            }
        })
    };

    // Wait a moment before subscribing
    println!("Waiting for connection to stabilize...");
    log_to_file(
        &log_filename,
        "Waiting 3 seconds for connection to stabilize",
    )
    .await?;
    sleep(Duration::from_secs(3)).await;

    // Create subscription message
    let wss_id =
        env::var("ORDERLY_WSS_ID").unwrap_or_else(|_| "evm_liquidation_stream".to_string());
    let subscription_msg = json!({
        "id": wss_id,
        "event": "subscribe",
        "topic": "liquidation"
    })
    .to_string();

    // Subscribe to liquidations feed
    println!("Subscribing to liquidations stream...");
    log_to_file(
        &log_filename,
        &format!("Sending subscription: {}", subscription_msg),
    )
    .await?;

    if let Err(e) = tx.send(Message::Text(subscription_msg)).await {
        log_to_file(
            &log_filename,
            &format!("Failed to send subscription to channel: {}", e),
        )
        .await?;
        return Err(e.into());
    }

    // No need for a separate ping task, as the server sends regular pings
    // We'll respond to those instead of sending our own

    // Process incoming messages
    println!("Waiting for messages... Press Ctrl+C to exit");
    log_to_file(&log_filename, "Waiting for messages...").await?;

    // Setup Ctrl+C handler for shutdown
    let shutdown = Arc::new(tokio::sync::Notify::new());
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            shutdown_clone.notify_one();
        }
    });

    let tx_for_pong = tx.clone();

    // Main loop for processing messages
    loop {
        tokio::select! {
            Some(msg) = read.next() => {
                match msg {
                    Ok(msg) => {
                        if let Message::Text(text) = &msg {
                            println!("Received: {}", text);
                            log_to_file(&log_filename, &format!("RECEIVED: {}", text)).await?;

                            // Respond to server pings with proper format
                            if text.contains("\"event\":\"ping\"") {
                                // Parse the timestamp from the ping
                                if let Some(ts_start) = text.find("\"ts\":") {
                                    if let Some(ts_end) = text[ts_start..].find("}") {
                                        let ts_str = &text[ts_start + 5..ts_start + ts_end];

                                        // Create proper pong response
                                        let pong_msg = json!({
                                            "event": "pong",
                                            "ts": ts_str.parse::<u64>().unwrap_or(0)
                                        }).to_string();

                                        println!("Received ping, sending proper pong format");
                                        log_to_file(&log_filename, &format!("Sending correctly formatted pong: {}", pong_msg)).await?;

                                        if let Err(e) = tx_for_pong.send(Message::Text(pong_msg)).await {
                                            log_to_file(&log_filename, &format!("Failed to send pong: {}", e)).await?;
                                        }
                                    }
                                }
                            }

                            // Process liquidation data if received
                            if text.contains("\"topic\":\"liquidation\"") && text.contains("\"data\":") {
                                println!("🚨 LIQUIDATION EVENT DETECTED 🚨");
                                log_to_file(&log_filename, "LIQUIDATION EVENT DETECTED").await?;
                            }
                        } else {
                            let msg_type = match &msg {
                                Message::Text(_) => "text",
                                Message::Binary(_) => "binary",
                                Message::Ping(_) => "ping",
                                Message::Pong(_) => "pong",
                                Message::Close(_) => "close",
                                Message::Frame(_) => "frame",
                            };
                            println!("Received {} message", msg_type);
                            log_to_file(&log_filename, &format!("Received {} message", msg_type)).await?;

                            // Handle WebSocket protocol messages
                            match msg {
                                Message::Ping(data) => {
                                    if let Err(e) = tx_for_pong.send(Message::Pong(data)).await {
                                        log_to_file(&log_filename, &format!("Failed to send pong: {}", e)).await?;
                                    }
                                },
                                Message::Close(_) => {
                                    println!("Server closed connection");
                                    log_to_file(&log_filename, "Server closed connection").await?;
                                    break;
                                },
                                _ => {}
                            }
                        }
                    },
                    Err(e) => {
                        println!("Error reading message: {}", e);
                        log_to_file(&log_filename, &format!("Error reading message: {}", e)).await?;
                        break;
                    }
                }
            },
            // Check for shutdown notification
            _ = shutdown.notified() => {
                println!("Received shutdown signal");
                log_to_file(&log_filename, "Shutdown requested").await?;
                break;
            }
        }
    }

    // Clean up
    println!("Closing connection...");
    log_to_file(&log_filename, "Closing connection").await?;

    // Abort the write task
    write_task.abort();

    // Send close message
    let _ = tx.send(Message::Close(None)).await;

    // Wait for task to complete
    let _ = write_task.await;

    println!("Connection closed. Log file: {}", log_filename);
    log_to_file(&log_filename, "Session ended").await?;

    Ok(())
}
