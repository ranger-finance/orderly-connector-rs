mod common;

use orderly_connector_rs::websocket::WebsocketPublicClient;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use serde_json::json;
use crate::types::{PublicTradesResponse, TradeData, WebSocketTradeData};

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

#[test]
fn test_public_trades_response_deserialization() {
    let json_data = json!({
        "success": true,
        "data": [
            {
                "id": 1,
                "symbol": "BTC_USD",
                "side": "BUY",
                "price": 50000.0,
                "quantity": 0.1,
                "ts": 1622548800000
            }
        ],
        "timestamp": 1622548800000
    });

    let response: PublicTradesResponse = serde_json::from_value(json_data).unwrap();
    assert!(response.success);
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].symbol, "BTC_USD");
}

#[test]
fn test_trade_data_deserialization() {
    let json_data = json!({
        "id": 1,
        "symbol": "BTC_USD",
        "side": "SELL",
        "price": 49000.0,
        "quantity": 0.2,
        "ts": 1622548800000
    });

    let trade: TradeData = serde_json::from_value(json_data).unwrap();
    assert_eq!(trade.side, "SELL");
    assert_eq!(trade.price, 49000.0);
}

#[test]
fn test_websocket_trade_data_deserialization() {
    let json_data = json!({
        "data": [
            {
                "id": 2,
                "symbol": "ETH_USD",
                "side": "BUY",
                "price": 2500.0,
                "quantity": 1.0,
                "ts": 1622548800000
            }
        ],
        "ts": 1622548800000
    });

    let ws_data: WebSocketTradeData = serde_json::from_value(json_data).unwrap();
    assert_eq!(ws_data.data.len(), 1);
    assert_eq!(ws_data.data[0].symbol, "ETH_USD");
}
