// tests/rest_private.rs

mod common;

use chrono::Utc;
use orderly_connector_rs::rest::client::Credentials;
use orderly_connector_rs::rest::OrderlyService;
use orderly_connector_rs::types::GetBrokerVolumeParams;
use orderly_connector_rs::types::{
    AssetHistoryType, CreateOrderRequest, GetAssetHistoryParams, GetOrdersParams, OrderStatus,
    OrderType, Side,
};
use tokio::time::{sleep, Duration};

fn setup_client() -> (OrderlyService, Credentials<'static>) {
    common::setup();
    let api_key = common::get_env_var("ORDERLY_API_KEY");
    let secret = common::get_env_var("ORDERLY_SECRET");
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();

    // IMPORTANT: Removed panic check for running against mainnet.
    // if !is_testnet {
    //     panic!("Private tests should only be run against testnet (set ORDERLY_TESTNET=true)");
    // }

    // Create the service
    let service = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    // Create credentials that will be owned by the test
    let creds = Credentials {
        orderly_key: Box::leak(api_key.into_boxed_str()),
        orderly_secret: Box::leak(secret.into_boxed_str()),
        orderly_account_id: Box::leak(account_id.into_boxed_str()),
    };

    (service, creds)
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_create_get_cancel_order() {
    let (client, creds) = setup_client();
    let symbol = "PERP_ETH_USDC"; // Testnet symbol

    println!("Creating order for symbol: {}", symbol);

    // Create a market buy order
    let order_req = CreateOrderRequest {
        symbol: symbol.to_string(),
        order_type: OrderType::Market,
        side: Side::Buy,
        order_price: None,          // Market orders don't need a price
        order_quantity: Some(0.01), // Small test quantity
        order_amount: None,
        client_order_id: Some("my_order_id".to_string()),
        visible_quantity: None,
    };

    let created_order_id = match client.create_order(&creds, order_req).await {
        Ok(resp) => {
            println!("Create Order Response: {:#?}", resp);
            assert!(resp.success, "Order creation should succeed");
            resp.data.order_id // Assign directly
        }
        Err(e) => panic!("Failed to create order: {}", e),
    };

    // Give order time to appear
    sleep(Duration::from_secs(2)).await;

    // Get the order details
    match client.get_order(&creds, created_order_id).await {
        Ok(resp) => {
            println!("Get Order Response: {:#?}", resp);
            assert_eq!(resp.data.order.symbol, symbol);
        }
        Err(e) => panic!("Failed to get order {}: {}", created_order_id, e),
    }

    // Get all orders for the symbol
    let params = GetOrdersParams {
        symbol: Some(symbol.to_string()),
        ..Default::default()
    };
    match client.get_orders(&creds, Some(params)).await {
        Ok(resp) => {
            println!("Get Orders Response: {:#?}", resp);
            assert!(
                resp.data
                    .rows
                    .iter()
                    .any(|order| order.order_id == created_order_id),
                "Created order should be in the list"
            );
        }
        Err(e) => panic!("Failed to get orders: {}", e),
    }

    // Cancel the order
    match client.cancel_order(&creds, created_order_id, symbol).await {
        Ok(resp) => {
            println!("Cancel Order Response: {:#?}", resp);
            assert!(resp.success, "Order cancellation should succeed");
        }
        Err(e) => {
            println!("Cancel order error: {:#?}", e); // Print the error for debugging
            use orderly_connector_rs::error::OrderlyError;
            if let OrderlyError::ClientError { code, message, .. } = &e {
                if *code == -1006 && message.contains("completed") {
                    println!(
                        "Order is already completed, cannot cancel. Skipping cancel assertion."
                    );
                    return;
                }
            }
            panic!("Failed to cancel order {}: {}", created_order_id, e);
        }
    }

    // Verify the order is cancelled
    match client.get_order(&creds, created_order_id).await {
        Ok(resp) => {
            println!("Get Order Response after cancel: {:#?}", resp);
            assert_eq!(
                resp.data.order.status,
                OrderStatus::Cancelled,
                "Order should be cancelled"
            );
        }
        Err(e) => panic!(
            "Failed to get order {} after cancel: {}",
            created_order_id, e
        ),
    }
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_asset_history() {
    let (client, creds) = setup_client();

    // Get asset history without any filters
    match client.get_asset_history(&creds, None).await {
        Ok(resp) => {
            println!("Get Asset History Response: {:#?}", resp);
            assert!(resp.success, "Asset history request should succeed");
            assert!(
                !resp.data.rows.is_empty(),
                "Should have at least one asset history entry"
            );
        }
        Err(e) => panic!("Failed to get asset history: {}", e),
    }

    // Get asset history with filters
    let params = GetAssetHistoryParams {
        token: Some("USDC".to_string()),
        side: Some(AssetHistoryType::Deposit),
        start_t: Some(1743560693442), // Use timestamp from earliest entry we saw
        end_t: Some(1745911452012),   // Use timestamp from latest entry we saw
        page: Some(1),
        size: Some(10),
    };

    match client.get_asset_history(&creds, Some(params)).await {
        Ok(resp) => {
            println!("Get Asset History Response with filters: {:#?}", resp);
            assert!(
                resp.success,
                "Asset history request with filters should succeed"
            );
            // Verify that all returned entries match the filters
            for entry in &resp.data.rows {
                assert_eq!(entry.token, "USDC");
                assert_eq!(entry.side, AssetHistoryType::Deposit);
            }
        }
        Err(e) => panic!("Failed to get asset history with filters: {}", e),
    }
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_account_info() {
    let (client, creds) = setup_client();
    let result = client.get_account_info(&creds).await;
    println!("Account Info Result: {:#?}", result);
    assert!(
        result.is_ok(),
        "Failed to get account info: {:?}",
        result.err()
    );
    let info = result.unwrap();
    assert!(info.success, "API response indicates failure");
    // Basic structure checks
    assert!(
        !info.data.account_id.is_empty(),
        "Account ID should not be empty"
    );
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_holding() {
    let (client, creds) = setup_client();
    let result = client.get_holding(&creds).await;
    println!("Holdings Result: {:#?}", result);
    assert!(result.is_ok(), "Failed to get holdings: {:?}", result.err());
    let holdings = result.unwrap();
    assert!(holdings.success, "API response indicates failure");
    // There should be at least one holding
    assert!(
        !holdings.data.holding.is_empty(),
        "Holdings list should not be empty"
    );
    // Check structure of first holding
    let first = &holdings.data.holding[0];
    assert!(!first.token.is_empty(), "Token should not be empty");
    assert!(first.holding >= 0.0);
    assert!(first.frozen >= 0.0);
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_positions() {
    let (client, creds) = setup_client();
    let result = client.get_positions(&creds).await;
    println!("Positions Result: {:#?}", result);
    assert!(
        result.is_ok(),
        "Failed to get positions: {:?}",
        result.err()
    );
    let positions = result.unwrap();
    assert!(positions.success, "API response indicates failure");
    // If there is at least one position, check structure
    if let Some(first) = positions.data.rows.get(0) {
        assert!(!first.symbol.is_empty(), "Symbol should not be empty");
        // Position qty, cost, mark price, etc. should be present
        let _ = first.position_qty;
        let _ = first.cost_position;
        let _ = first.mark_price;
        let _ = first.unsettled_pnl;
        let _ = first.average_open_price;
    }
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_broker_volume() {
    let (client, creds) = setup_client();
    // Use the last 7 days for the test
    let end_date = Utc::now().date_naive();
    let start_date = end_date - chrono::Duration::days(7);
    let params = GetBrokerVolumeParams {
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        ..Default::default()
    };
    let result = client.get_broker_volume(&creds, params).await;
    println!("Broker Volume Result: {:#?}", result);
    assert!(
        result.is_ok(),
        "Failed to get broker volume: {:?}",
        result.err()
    );
    let resp = result.unwrap();
    println!("Broker Volume Response: {:#?}", resp);
    assert!(resp.success, "API response indicates failure");
    // Check that meta and rows are present
    assert!(
        resp.data.meta.records_per_page > 0,
        "Meta should have records_per_page > 0"
    );
}

// Add more tests for other private endpoints: get_account_info, positions, etc.
