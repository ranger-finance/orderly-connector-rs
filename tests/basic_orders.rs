use mockito::{self, Server};
use orderly_connector_rs::{
    rest::{client::Credentials, OrderlyService},
    types::{OrderStatus, OrderType, Side},
};
use serde_json::json;

// Helper function to create test credentials
fn test_credentials() -> Credentials<'static> {
    Credentials {
        orderly_key: "test_key",
        orderly_secret: "11111111111111111111111111111111",
        orderly_account_id: "test_account",
    }
}

#[tokio::test]
async fn test_market_order_creation() {
    let mut server = Server::new_async().await;

    // Mock successful market order response
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "order_id": 12345,
            "client_order_id": null
        }
    });

    let _m = server
        .mock("POST", "/v1/order")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Json(json!({
            "symbol": "PERP_ETH_USDC",
            "order_type": "MARKET",
            "side": "BUY",
            "order_quantity": 0.01
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let result = client
        .create_order(
            &creds,
            orderly_connector_rs::types::CreateOrderRequest {
                symbol: "PERP_ETH_USDC",
                order_type: OrderType::Market,
                side: Side::Buy,
                order_price: None,
                order_quantity: Some(0.01),
                order_amount: None,
                client_order_id: None,
                visible_quantity: None,
            },
        )
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.data.order_id, 12345);
}

#[tokio::test]
async fn test_limit_order_creation() {
    let mut server = Server::new_async().await;

    // Mock successful limit order response
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "order_id": 12346,
            "client_order_id": null
        }
    });

    let _m = server
        .mock("POST", "/v1/order")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Json(json!({
            "symbol": "PERP_ETH_USDC",
            "order_type": "LIMIT",
            "side": "SELL",
            "order_price": 2000.0,
            "order_quantity": 0.01
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let result = client
        .create_order(
            &creds,
            orderly_connector_rs::types::CreateOrderRequest {
                symbol: "PERP_ETH_USDC",
                order_type: OrderType::Limit,
                side: Side::Sell,
                order_price: Some(2000.0),
                order_quantity: Some(0.01),
                order_amount: None,
                client_order_id: None,
                visible_quantity: None,
            },
        )
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.data.order_id, 12346);
}

#[tokio::test]
async fn test_order_cancellation() {
    let mut server = Server::new_async().await;

    // Mock successful cancellation response
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "status": "CANCEL_SENT"
        }
    });

    let _m = server
        .mock("DELETE", "/v1/order?order_id=12345&symbol=PERP_ETH_USDC")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let result = client.cancel_order(&creds, 12345, "PERP_ETH_USDC").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.data.status, "CANCEL_SENT");
}

#[tokio::test]
async fn test_order_status_monitoring() {
    let mut server = Server::new_async().await;

    // Mock order status responses for monitoring
    let mock_response = |status: OrderStatus| {
        let status_str = match status {
            OrderStatus::New => "NEW",
            OrderStatus::PartialFilled => "PARTIAL_FILLED",
            OrderStatus::Filled => "FILLED",
            OrderStatus::Cancelled => "CANCELLED",
            OrderStatus::Rejected => "REJECTED",
            OrderStatus::Expired => "EXPIRED",
            OrderStatus::Accepted => "ACCEPTED",
        };

        json!({
            "success": true,
            "timestamp": 1677721600123_u64,
            "data": {
                "order_id": 12345,
                "symbol": "PERP_ETH_USDC",
                "side": "BUY",
                "order_type": "MARKET",
                "order_price": null,
                "order_quantity": 0.01,
                "order_amount": null,
                "status": status_str,
                "executed_quantity": match status {
                    OrderStatus::PartialFilled => 0.005,
                    OrderStatus::Filled => 0.01,
                    _ => 0.0
                },
                "executed_price": 1900.0,
                "created_time": 1677721600000_u64,
                "updated_time": 1677721600000_u64
            }
        })
    };

    // Set up mock responses in sequence
    let _m1 = server
        .mock("GET", "/v1/order/12345")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response(OrderStatus::New).to_string())
        .expect(1)
        .create_async()
        .await;

    let _m2 = server
        .mock("GET", "/v1/order/12345")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response(OrderStatus::PartialFilled).to_string())
        .expect(1)
        .create_async()
        .await;

    let _m3 = server
        .mock("GET", "/v1/order/12345")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response(OrderStatus::Filled).to_string())
        .expect(1)
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    // First check - New
    let result = client.get_order(&creds, 12345).await;
    match &result {
        Ok(resp) => println!("First check response: {:?}", resp),
        Err(e) => println!("First check error: {}", e),
    }
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data.order.status, OrderStatus::New);

    // Second check - PartialFilled
    let result = client.get_order(&creds, 12345).await;
    match &result {
        Ok(resp) => println!("Second check response: {:?}", resp),
        Err(e) => println!("Second check error: {}", e),
    }
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().data.order.status,
        OrderStatus::PartialFilled
    );

    // Third check - Filled
    let result = client.get_order(&creds, 12345).await;
    match &result {
        Ok(resp) => println!("Third check response: {:?}", resp),
        Err(e) => println!("Third check error: {}", e),
    }
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data.order.status, OrderStatus::Filled);
}

#[tokio::test]
async fn test_error_handling() {
    let mut server = Server::new_async().await;

    // Mock error response
    let mock_response = json!({
        "success": false,
        "timestamp": 1677721600123_u64,
        "code": 100001,
        "message": "Invalid order parameters"
    });

    let _m = server
        .mock("POST", "/v1/order")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .match_header("orderly-timestamp", mockito::Matcher::Any)
        .match_header("orderly-signature", mockito::Matcher::Any)
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let result = client
        .create_order(
            &creds,
            orderly_connector_rs::types::CreateOrderRequest {
                symbol: "INVALID_SYMBOL",
                order_type: OrderType::Market,
                side: Side::Buy,
                order_price: None,
                order_quantity: Some(0.0), // Invalid quantity
                order_amount: None,
                client_order_id: None,
                visible_quantity: None,
            },
        )
        .await;

    assert!(result.is_err());
    match result {
        Err(e) => {
            assert!(e.to_string().contains("Invalid order parameters"));
        }
        _ => panic!("Expected error"),
    }
}
