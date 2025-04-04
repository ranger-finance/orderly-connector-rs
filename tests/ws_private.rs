// tests/ws_private.rs

mod common;

use orderly_connector_rs::websocket::WebsocketPrivateClient;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::test]
#[ignore] // Ignored by default (requires network + credentials)
async fn test_private_ws_connect_subscribe_unsubscribe() {
    common::setup();
    let api_key = common::get_env_var("ORDERLY_API_KEY");
    let secret = common::get_env_var("ORDERLY_SECRET");
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();

    // Channel to signal message reception
    let (tx, mut rx) = mpsc::channel::<String>(10);
    // Flag to signal connection close
    let closed = Arc::new(AtomicBool::new(false));
    let closed_clone = closed.clone();

    let message_handler = Arc::new(move |msg: String| {
        // Simple check for common private message types
        if msg.contains("executionreport") || msg.contains("balance") {
            println!(
                "Received private message snippet: {}...",
                msg.chars().take(100).collect::<String>()
            );
            // Try sending, ignore error if channel is full/closed
            let _ = tx.try_send(msg);
        }
    });

    let close_handler = Arc::new(move || {
        closed_clone.store(true, Ordering::SeqCst);
        println!("Private WS Connection Closed (test handler).");
    });

    // Connect
    let connect_result = WebsocketPrivateClient::connect(
        api_key,
        secret,
        account_id.clone(),
        is_testnet,
        message_handler,
        close_handler,
    )
    .await;

    assert!(connect_result.is_ok(), "Private WS connection failed");
    let client = connect_result.unwrap();
    println!("Private WS Connected.");

    // Subscribe to execution reports
    let sub_exec_result = client.subscribe_execution_reports().await;
    assert!(
        sub_exec_result.is_ok(),
        "Failed to subscribe to execution reports"
    );
    println!("Subscribed to execution reports.");

    // Subscribe to balance updates
    let sub_bal_result = client.subscribe_balance().await;
    assert!(
        sub_bal_result.is_ok(),
        "Failed to subscribe to balance updates"
    );
    println!("Subscribed to balance updates.");

    // Wait for at least one relevant message (execution report or balance)
    // This might require triggering an action (like placing an order via REST)
    // in a real scenario, but here we just wait.
    println!("Waiting for private message (execution report or balance)...");
    match timeout(Duration::from_secs(20), rx.recv()).await {
        Ok(Some(msg)) => {
            println!("Received private message: {}", msg);
            // Assert basic structure if needed, e.g., contains expected fields
            assert!(msg.contains("topic") && msg.contains("data"));
        }
        Ok(None) => println!("Warning: Message channel closed before receiving expected private message. Manual trigger might be needed."),
        Err(_) => println!("Warning: Timeout waiting for private WS message. Manual trigger might be needed."),
        // Not panicking here as receiving might depend on external actions
    }

    // Unsubscribe
    let unsub_exec_result = client.unsubscribe_execution_reports().await;
    assert!(
        unsub_exec_result.is_ok(),
        "Failed to unsubscribe from execution reports"
    );
    println!("Unsubscribed from execution reports.");

    let unsub_bal_result = client.unsubscribe_balance().await;
    assert!(
        unsub_bal_result.is_ok(),
        "Failed to unsubscribe from balance"
    );
    println!("Unsubscribed from balance updates.");

    // Give time for potential confirmation/cleanup
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Stop
    client.stop().await;
    println!("Private WS Client stopped.");

    // Check if close handler was called (allow some time)
    tokio::time::sleep(Duration::from_secs(2)).await;
    // assert!(closed.load(Ordering::SeqCst), "Close handler was not called after stop");
    // See note in ws_public test regarding this assertion.
}
