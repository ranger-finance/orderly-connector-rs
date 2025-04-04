// tests/rest_public.rs

// Import the common setup function
mod common;

use orderly_connector_rs::rest::Client;

#[tokio::test]
#[ignore] // Ignored by default as it requires network access and credentials
async fn test_get_system_status() {
    common::setup(); // Load .env variables
    let api_key = common::get_env_var("ORDERLY_API_KEY");
    let secret = common::get_env_var("ORDERLY_SECRET");
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();

    let client = Client::new(api_key, secret, account_id, is_testnet, None)
        .expect("Failed to create REST client");

    let result = client.get_system_status().await;
    println!("System Status Result: {:?}", result);
    assert!(result.is_ok());
    let status_resp = result.unwrap();
    assert!(status_resp["success"].as_bool().unwrap_or(false));
    assert!(status_resp["data"].is_object());
    assert_eq!(status_resp["data"]["status"].as_i64(), Some(0));
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_exchange_info_all() {
    common::setup();
    let api_key = common::get_env_var("ORDERLY_API_KEY");
    let secret = common::get_env_var("ORDERLY_SECRET");
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();

    let client = Client::new(api_key, secret, account_id, is_testnet, None)
        .expect("Failed to create REST client");

    let result = client.get_exchange_info(None).await;
    println!("Exchange Info (All) Result: {:?}", result);
    assert!(result.is_ok());
    let info_resp = result.unwrap();
    assert!(info_resp["success"].as_bool().unwrap_or(false));
    assert!(info_resp["data"].is_object());
    assert!(info_resp["data"]["symbols"]
        .as_array()
        .map_or(false, |arr| !arr.is_empty()));
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_exchange_info_specific() {
    common::setup();
    let api_key = common::get_env_var("ORDERLY_API_KEY");
    let secret = common::get_env_var("ORDERLY_SECRET");
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();
    let symbol = "PERP_ETH_USDC"; // Assumes this symbol exists on testnet/mainnet

    let client = Client::new(api_key, secret, account_id, is_testnet, None)
        .expect("Failed to create REST client");

    let result = client.get_exchange_info(Some(symbol)).await;
    println!("Exchange Info ({}) Result: {:?}", symbol, result);
    assert!(result.is_ok());
    let info_resp = result.unwrap();
    assert!(info_resp["success"].as_bool().unwrap_or(false));
    assert!(info_resp["data"].is_object());
    assert_eq!(info_resp["data"]["symbol"].as_str(), Some(symbol));
}

// Add more tests for other public endpoints like get_futures_info...
