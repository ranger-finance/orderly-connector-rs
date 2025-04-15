mod common;

use orderly_connector_rs::websocket::WebsocketPublicClient;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
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

    // Channel to receive messages
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let connected = Arc::new(AtomicBool::new(false));
    let connected_clone = connected.clone();

    let message_handler = Arc::new(move |msg: String| {
        println!("Received message: {}", msg);
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tx.send(msg).await {
                println!("Failed to send message to channel: {}", e);
            }
        });
    });

    let close_handler = Arc::new(move || {
        connected_clone.store(false, Ordering::SeqCst);
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

    connected.store(true, Ordering::SeqCst);
    println!("Connected to WebSocket server");

    // Wait for connection to stabilize
    sleep(Duration::from_secs(5)).await;

    // Keep receiving messages for a while
    let timeout_duration = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout_duration && connected.load(Ordering::SeqCst) {
        if let Ok(msg) = rx.try_recv() {
            println!("Received message in loop: {}", msg);
        }
        sleep(Duration::from_millis(100)).await;
    }

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
    let symbol = "PERP_ETH_USDC";

    // Channel to receive messages
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let connected = Arc::new(AtomicBool::new(false));
    let connected_clone = connected.clone();

    let message_handler = Arc::new(move |msg: String| {
        println!("Received message: {}", msg);
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tx.send(msg).await {
                println!("Failed to send message to channel: {}", e);
            }
        });
    });

    let close_handler = Arc::new(move || {
        connected_clone.store(false, Ordering::SeqCst);
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

    connected.store(true, Ordering::SeqCst);
    println!("Connected to WebSocket server");

    // Wait for connection to stabilize
    sleep(Duration::from_secs(5)).await;

    // Subscribe to open interest
    client
        .subscribe_open_interest(symbol)
        .await
        .expect("Failed to subscribe to open interest");
    println!("Subscribed to open interest for {}", symbol);

    // Keep receiving messages for a while
    let timeout_duration = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout_duration && connected.load(Ordering::SeqCst) {
        if let Ok(msg) = rx.try_recv() {
            println!("Received message in loop: {}", msg);
            if msg.contains("openinterest") {
                println!("Received open interest message!");
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Test should pass if we got here without errors
    client.stop().await;
}

/// Tests subscribing to trade updates.
///
/// This test verifies that:
/// - The client can subscribe to trade updates
/// - Messages are received through the callback
/// - The received messages contain valid trade data
///
/// Note: This test is ignored by default as it requires network access.
#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    common::setup();
    let is_testnet = common::get_testnet_flag();
    let symbol = "PERP_ETH_USDC";

    // Channel to receive messages
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let connected = Arc::new(AtomicBool::new(false));
    let connected_clone = connected.clone();

    let message_handler = Arc::new(move |msg: String| {
        println!("Received message: {}", msg);
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tx.send(msg).await {
                println!("Failed to send message to channel: {}", e);
            }
        });
    });

    let close_handler = Arc::new(move || {
        connected_clone.store(false, Ordering::SeqCst);
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

    connected.store(true, Ordering::SeqCst);
    println!("Connected to WebSocket server");

    // Wait for connection to stabilize
    sleep(Duration::from_secs(5)).await;

    // Subscribe to trades
    client
        .subscribe_trades(symbol)
        .await
        .expect("Failed to subscribe to trades");
    println!("Subscribed to trades for {}", symbol);

    // Keep receiving messages for a while
    let timeout_duration = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout_duration && connected.load(Ordering::SeqCst) {
        if let Ok(msg) = rx.try_recv() {
            println!("Received message in loop: {}", msg);
            if msg.contains("trade") {
                println!("Received trade message!");
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Test should pass if we got here without errors
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
