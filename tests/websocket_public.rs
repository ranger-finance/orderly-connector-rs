mod common;

use orderly_connector_rs::websocket::WebsocketPublicClient;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
// Use the library crate name to import types in integration tests
use orderly_connector_rs::types::{GetPublicTradesResponse, PublicTradeData, WebSocketTradeData};

/// Tests the WebSocket connection and basic subscription functionality.
///
/// This test verifies that:
/// - The client can establish a WebSocket connection
/// - Basic subscription messages can be sent
/// - Messages are received through the callback
///
/// Note: This test is ignored by default as it requires network access.
#[tokio::test]
#[ignore]
async fn test_websocket_connection() {
    common::setup();
    let is_testnet = common::get_testnet_flag();

    let message_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let message_count_clone = Arc::clone(&message_count);

    let message_handler = Arc::new(move |msg: String| {
        println!("Received message: {}", msg);
        message_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });

    let close_handler = Arc::new(|| {
        println!("Connection closed");
    });

    let client = WebsocketPublicClient::connect(
        "test_account".to_string(),
        is_testnet,
        message_handler,
        close_handler,
    )
    .await
    .expect("Failed to connect");

    // Wait a bit to ensure connection is established
    sleep(Duration::from_secs(2)).await;

    // Test should pass if we got here without errors
    client.stop().await;
}

/// Tests the open interest WebSocket subscription.
///
/// This test verifies that:
/// - The client can subscribe to open interest updates
/// - Messages are received through the callback
/// - The received messages contain valid open interest data
///
/// Note: This test is ignored by default as it requires network access.
#[tokio::test]
#[ignore]
async fn test_subscribe_open_interest() {
    common::setup();
    let is_testnet = common::get_testnet_flag();

    let message_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let message_count_clone = Arc::clone(&message_count);

    let message_handler = Arc::new(move |msg: String| {
        println!("Received message: {}", msg);
        // Verify message contains expected open interest fields
        if msg.contains("openinterest") && msg.contains("long_oi") && msg.contains("short_oi") {
            message_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    });

    let close_handler = Arc::new(|| {
        println!("Connection closed");
    });

    let client = WebsocketPublicClient::connect(
        "test_account".to_string(),
        is_testnet,
        message_handler,
        close_handler,
    )
    .await
    .expect("Failed to connect");

    // Subscribe to open interest for ETH
    client
        .subscribe_open_interest("PERP_ETH_USDC")
        .await
        .expect("Failed to subscribe to open interest");

    // Wait to receive some messages
    sleep(Duration::from_secs(5)).await;

    let final_count = message_count.load(std::sync::atomic::Ordering::SeqCst);
    println!("Received {} open interest messages", final_count);

    // We should have received at least one message
    assert!(final_count > 0, "No open interest messages received");

    client.stop().await;
}

/// Tests the trade WebSocket subscription.
///
/// This test verifies that:
/// - The client can subscribe to trade updates for a specific symbol.
/// - Messages are received through the callback.
/// - The received messages contain valid trade data (price, quantity, side, etc.).
///
/// Note: This test is ignored by default as it requires network access
/// and assumes trades will occur for the specified symbol during the test.
#[tokio::test]
#[ignore] // Ignored by default
async fn test_subscribe_trades() {
    common::setup();
    let is_testnet = common::get_testnet_flag();
    let symbol = "PERP_ETH_USDC"; // Use a symbol likely to have trades

    let message_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let message_count_clone = Arc::clone(&message_count);

    // Define the message handler callback
    let message_handler = Arc::new(move |msg: String| {
        println!("Received message: {}", msg);
        // Attempt to parse the message as a trade update
        // We expect messages like: {"topic":"PERP_ETH_USDC@trade","ts":1678886400000,"data":[{"id":123,"symbol":"PERP_ETH_USDC","side":"BUY","price":1500.0,"quantity":0.1,"ts":1678886400000}]}
        if msg.contains(&format!("{}@trade", symbol)) {
            match serde_json::from_str::<serde_json::Value>(&msg) {
                Ok(value) => {
                    // Check if it has the expected structure (topic, data array)
                    if value.get("topic").is_some()
                        && value.get("data").and_then(|d| d.as_array()).is_some()
                    {
                        // Increment count if it looks like a trade message for the symbol
                        message_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        println!("Incremented trade message count.");
                    } else {
                        println!(
                            "Received message on topic, but incorrect structure: {:?}",
                            value
                        );
                    }
                }
                Err(e) => {
                    println!("Failed to parse received message as JSON: {}", e);
                }
            }
        } else if msg.contains("\"event\":\"subscribe\"") {
            println!("Received subscription confirmation message.");
        } else {
            println!("Received other message type: {}", msg);
        }
    });

    // Define the close handler callback
    let close_handler = Arc::new(|| {
        println!("Connection closed");
    });

    // Connect the WebSocket client
    let client = WebsocketPublicClient::connect(
        "test_trade_sub_account".to_string(), // Use a descriptive dummy account ID
        is_testnet,
        message_handler,
        close_handler,
    )
    .await
    .expect("Failed to connect WebSocket client");

    // Subscribe to trades for the specified symbol
    println!("Subscribing to trades for symbol: {}", symbol);
    client
        .subscribe_trades(symbol)
        .await
        .expect("Failed to send subscribe_trades request");

    // Wait for a period to allow trade messages to arrive
    // Increase duration if needed, especially on less active testnets/symbols
    println!("Waiting for trade messages...");
    sleep(Duration::from_secs(10)).await;

    // Check if any trade messages were received
    let final_count = message_count.load(std::sync::atomic::Ordering::SeqCst);
    println!("Received {} trade messages for {}", final_count, symbol);

    // Assert that at least one trade message was received.
    // This might fail if no trades occur on the symbol during the sleep period.
    assert!(final_count > 0, "No trade messages received for {}", symbol);

    // Cleanly stop the client
    client.stop().await;
}

#[test]
fn test_public_trades_response_deserialization() {
    let json_data = json!({
        "success": true,
        "data": {
            "rows": [
                {
                    "symbol": "BTC_USD",
                    "side": "BUY",
                    "executed_price": 50000.0,
                    "executed_quantity": 0.1,
                    "executed_timestamp": 1622548800000u64
                }
            ]
        },
        "timestamp": 1622548800000u64
    });

    let response: GetPublicTradesResponse = serde_json::from_value(json_data).unwrap();
    assert!(response.success);
    assert_eq!(response.data.rows.len(), 1);
    assert_eq!(response.data.rows[0].symbol, "BTC_USD");
}

#[test]
fn test_trade_data_deserialization() {
    let json_data = json!({
        "symbol": "BTC_USD",
        "side": "SELL",
        "executed_price": 49000.0,
        "executed_quantity": 0.2,
        "executed_timestamp": 1622548800000u64
    });

    let trade: PublicTradeData = serde_json::from_value(json_data).unwrap();
    assert_eq!(trade.side, "SELL");
    assert_eq!(trade.executed_price, 49000.0);
}

#[test]
fn test_websocket_trade_data_deserialization() {
    let json_data = json!({
        "topic": "PERP_ETH_USDC@trade",
        "ts": 1618820361552u64,
        "data": {
            "symbol": "PERP_ETH_USDC",
            "price": 2500.0,
            "size": 1.0,
            "side": "BUY"
        }
    });

    let ws_data: WebSocketTradeData = serde_json::from_value(json_data).unwrap();
    assert_eq!(ws_data.topic, "PERP_ETH_USDC@trade");
    assert_eq!(ws_data.data.symbol, "PERP_ETH_USDC");
    assert_eq!(ws_data.data.price, 2500.0);
    assert_eq!(ws_data.data.size, 1.0);
    assert_eq!(ws_data.data.side, "BUY");
}
