// tests/rest_private.rs

mod common;

use orderly_connector_rs::rest::client::Credentials;
use orderly_connector_rs::rest::OrderlyService;
use orderly_connector_rs::types::{
    CreateOrderRequest, GetOrdersParams, OrderStatus, OrderType, Side,
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
        Err(e) => panic!("Failed to cancel order {}: {}", created_order_id, e),
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

// Add more tests for other private endpoints: get_account_info, positions, etc.
