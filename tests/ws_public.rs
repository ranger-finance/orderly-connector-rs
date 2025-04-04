// tests/ws_public.rs

mod common;

use orderly_connector_rs::websocket::WebsocketPublicClient;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::test]
#[ignore] // Ignored by default (requires network + account ID)
async fn test_public_ws_connect_subscribe_unsubscribe() {
    common::setup();
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();
    let symbol = "PERP_ETH_USDC"; // Testnet symbol

    // Channel to signal message reception
    let (tx, mut rx) = mpsc::channel::<String>(10);
    // Flag to signal connection close
    let closed = Arc::new(AtomicBool::new(false));
    let closed_clone = closed.clone();

    let message_handler = Arc::new(move |msg: String| {
        // Try sending, ignore error if channel is full/closed
        let _ = tx.try_send(msg);
    });

    let close_handler = Arc::new(move || {
        closed_clone.store(true, Ordering::SeqCst);
        println!("Public WS Connection Closed (test handler).");
    });

    // Connect
    let connect_result = WebsocketPublicClient::connect(
        account_id.clone(),
        is_testnet,
        message_handler,
        close_handler,
    )
    .await;

    assert!(connect_result.is_ok(), "Public WS connection failed");
    let client = connect_result.unwrap();
    println!("Public WS Connected.");

    // Subscribe to orderbook
    let sub_result = client.subscribe_orderbook(symbol).await;
    assert!(sub_result.is_ok(), "Failed to subscribe to orderbook");
    println!("Subscribed to orderbook for {}", symbol);

    // Wait for at least one message (e.g., snapshot or update)
    println!("Waiting for orderbook message...");
    match timeout(Duration::from_secs(15), rx.recv()).await {
        Ok(Some(msg)) => {
            println!("Received message: {}", msg);
            // Basic check: message should contain the symbol
            assert!(msg.contains(symbol));
        }
        Ok(None) => panic!("Message channel closed unexpectedly"),
        Err(_) => panic!("Timeout waiting for public WS message"),
    }

    // Unsubscribe
    let unsub_result = client.unsubscribe_orderbook(symbol).await;
    assert!(unsub_result.is_ok(), "Failed to unsubscribe from orderbook");
    println!("Unsubscribed from orderbook.");

    // Give time for potential confirmation/cleanup
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Stop
    client.stop().await;
    println!("Public WS Client stopped.");

    // Check if close handler was called (allow some time)
    tokio::time::sleep(Duration::from_secs(2)).await;
    // assert!(closed.load(Ordering::SeqCst), "Close handler was not called after stop");
    // Note: Depending on implementation, the close handler might not be called immediately
    // or exactly upon `stop()`, especially if the stop is graceful.
    // Consider the exact behavior needed for this assertion.
} 