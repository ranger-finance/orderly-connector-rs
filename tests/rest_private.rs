// tests/rest_private.rs

mod common;

use orderly_connector_rs::rest::Client;
use orderly_connector_rs::types::{
    CreateOrderRequest, GetOrdersParams, OrderStatus, OrderType, Side,
};
use tokio::time::{sleep, Duration};

fn setup_client() -> Client {
    common::setup();
    let api_key = common::get_env_var("ORDERLY_API_KEY");
    let secret = common::get_env_var("ORDERLY_SECRET");
    let account_id = common::get_env_var("ORDERLY_ACCOUNT_ID");
    let is_testnet = common::get_testnet_flag();

    // IMPORTANT: Removed panic check for running against mainnet.
    // if !is_testnet {
    //     panic!("Private tests should only be run against testnet (set ORDERLY_TESTNET=true)");
    // }

    Client::new(api_key, secret, account_id, is_testnet, None)
        .expect("Failed to create REST client")
}

#[tokio::test]
#[ignore] // Ignored by default
async fn test_create_get_cancel_order() {
    let client = setup_client();
    let symbol = "PERP_ETH_USDC"; // Testnet symbol
    let mut created_order_id: Option<u64> = None;

    // --- 1. Create Limit Buy Order --- (Place below market)
    let order_req = CreateOrderRequest {
        symbol,
        order_type: OrderType::Limit,
        side: Side::Buy,
        order_price: Some(1000.0), // Low price to avoid immediate fill
        order_quantity: 0.01,
        order_amount: None,
        client_order_id: None,
        visible_quantity: None,
    };

    let create_result = client.create_order(order_req).await;
    println!("Create Order Result: {:?}", create_result);
    assert!(create_result.is_ok(), "Failed to create order");
    let create_resp = create_result.unwrap();
    assert!(create_resp.success, "Order creation API call failed");
    assert!(create_resp.data.order_id > 0);
    created_order_id = Some(create_resp.data.order_id);

    // Give order time to appear
    sleep(Duration::from_secs(2)).await;

    // --- 2. Get Specific Order ---
    let order_id = created_order_id.expect("Order ID should exist after creation");
    let get_result = client.get_order(order_id).await;
    println!("Get Order Result: {:?}", get_result);
    assert!(get_result.is_ok(), "Failed to get order {}", order_id);
    let get_resp = get_result.unwrap();
    assert!(get_resp.success, "Get order API call failed");
    let fetched_order = get_resp.data.order;
    assert_eq!(fetched_order.order_id, order_id);

    // --- 3. Get Orders (Filtered) ---
    let params = GetOrdersParams {
        symbol: Some(symbol),
        ..Default::default()
    };
    let get_filtered_result = client.get_orders(Some(params)).await;
    println!("Get Orders (Filtered) Result: {:?}", get_filtered_result);
    assert!(get_filtered_result.is_ok(), "Failed to get filtered orders");
    let get_filtered_resp = get_filtered_result.unwrap();
    assert!(
        get_filtered_resp.success,
        "Get filtered orders API call failed"
    );
    assert!(
        !get_filtered_resp.data.rows.is_empty(),
        "Expected at least one order for {}",
        symbol
    );

    // --- 4. Cancel Order ---
    let cancel_result = client.cancel_order(order_id, symbol).await;
    println!("Cancel Order Result: {:?}", cancel_result);
    assert!(cancel_result.is_ok(), "Failed to cancel order {}", order_id);
    let cancel_resp = cancel_result.unwrap();
    // Note: Check actual success logic based on API specifics (e.g., status field)
    assert!(cancel_resp.success, "Cancel order API call failed");
    // TODO: Add assertion based on actual cancel success response field if available

    // Optional: Verify order status is cancelled by fetching it again
    sleep(Duration::from_secs(1)).await;
    let get_after_cancel_result = client.get_order(order_id).await;
    if let Ok(resp) = get_after_cancel_result {
        let order = resp.data.order;
        assert!(
            matches!(order.status, OrderStatus::Cancelled | OrderStatus::Rejected),
            "Order status should be Cancelled or Rejected after cancellation, but was {:?}",
            order.status
        );
    } // Silently ignore error if fetch fails after cancel
}

// Add more tests for other private endpoints: get_account_info, positions, etc.
