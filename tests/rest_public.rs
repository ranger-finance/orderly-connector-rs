// tests/rest_public.rs

// Import the common setup function
mod common;

use orderly_connector_rs::rest::OrderlyService;

#[tokio::test]
#[ignore] // Ignored by default as it requires network access and credentials
async fn test_get_system_status() {
    common::setup(); // Load .env variables
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_system_status().await;
    println!("System Status Result: {:?}", result);
    assert!(result.is_ok());
    let status_resp = result.unwrap();
    assert!(status_resp["success"].as_bool().unwrap_or(false));
    assert!(status_resp["data"].is_object());
    assert_eq!(status_resp["data"]["status"].as_i64(), Some(0));
}

#[tokio::test]
#[ignore] // Ignored by default as it requires network access
async fn test_get_exchange_info_all() {
    common::setup(); // Load .env variables
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_exchange_info(None).await;
    println!("Exchange Info (All) Result: {:?}", result);
    assert!(
        result.is_ok(),
        "Failed to get exchange info: {:?}",
        result.err()
    );

    let info_resp = result.unwrap();
    assert!(
        info_resp["success"].as_bool().unwrap_or(false),
        "API response indicates failure"
    );
    assert!(
        info_resp["data"].is_object(),
        "Response data should be an object"
    );

    // Validate symbols array exists and has content
    let symbols = info_resp["data"]["symbols"]
        .as_array()
        .expect("symbols should be an array");
    assert!(!symbols.is_empty(), "symbols array should not be empty");

    // Validate structure of first symbol
    let first_symbol = &symbols[0];
    assert!(
        first_symbol["symbol"].is_string(),
        "symbol should be a string"
    );
    assert!(
        first_symbol["quote_token"].is_string(),
        "quote_token should be a string"
    );
    assert!(
        first_symbol["base_token"].is_string(),
        "base_token should be a string"
    );
    assert!(
        first_symbol["price_precision"].is_number(),
        "price_precision should be a number"
    );
    assert!(
        first_symbol["quantity_precision"].is_number(),
        "quantity_precision should be a number"
    );
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

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_exchange_info(Some(symbol)).await;
    println!("Exchange Info ({}) Result: {:?}", symbol, result);
    assert!(result.is_ok());
    let info_resp = result.unwrap();
    assert!(info_resp["success"].as_bool().unwrap_or(false));
    assert!(info_resp["data"].is_object());
    assert_eq!(info_resp["data"]["symbol"].as_str(), Some(symbol));
}

// Add more tests for other public endpoints like get_futures_info...
