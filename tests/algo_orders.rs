use orderly_connector_rs::{
    rest::Client,
    types::{AlgoOrderType, CreateAlgoOrderRequest, Side, GetAlgoOrdersParams},
};
use mockito::mock;
use rust_decimal_macros::dec;
use serde_json::json;

#[tokio::test]
async fn test_create_algo_order() {
    let mut server = mockito::Server::new();
    let mock_response = json!({
        "success": true,
        "data": {
            "algo_order_id": "123456",
            "client_order_id": "my_stop_loss_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": "0.1",
            "trigger_price": "50000",
            "status": "NEW",
            "reduce_only": true,
            "created_time": 1677721600000,
            "updated_time": 1677721600000
        }
    });

    let _m = server.mock("POST", "/v1/algo-order")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = Client::new(
        &server.url(),
        Some("test_key"),
        Some("test_secret"),
        Some("test_account"),
    ).unwrap();

    let request = CreateAlgoOrderRequest {
        symbol: "PERP_BTC_USDC".to_string(),
        order_type: AlgoOrderType::StopMarket,
        side: Side::Sell,
        quantity: dec!(0.1),
        trigger_price: dec!(50000),
        limit_price: None,
        trailing_delta: None,
        client_order_id: Some("my_stop_loss_1".to_string()),
        reduce_only: Some(true),
    };

    let result = client.create_algo_order(request).await;
    assert!(result.is_ok());

    let order = result.unwrap();
    assert_eq!(order.algo_order_id, "123456");
    assert_eq!(order.symbol, "PERP_BTC_USDC");
    assert_eq!(order.order_type, AlgoOrderType::StopMarket);
    assert_eq!(order.side, Side::Sell);
}

#[tokio::test]
async fn test_cancel_algo_order() {
    let mut server = mockito::Server::new();
    let mock_response = json!({
        "success": true,
        "data": {
            "algo_order_id": "123456",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": "0.1",
            "trigger_price": "50000",
            "status": "CANCELLED",
            "reduce_only": true,
            "created_time": 1677721600000,
            "updated_time": 1677721700000
        }
    });

    let _m = server.mock("DELETE", "/v1/algo-order/PERP_BTC_USDC/123456")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = Client::new(
        &server.url(),
        Some("test_key"),
        Some("test_secret"),
        Some("test_account"),
    ).unwrap();

    let result = client.cancel_algo_order("PERP_BTC_USDC", "123456").await;
    assert!(result.is_ok());

    let order = result.unwrap();
    assert_eq!(order.algo_order_id, "123456");
    assert_eq!(order.status.to_string(), "CANCELLED");
}

#[tokio::test]
async fn test_cancel_algo_order_by_client_id() {
    let mut server = mockito::Server::new();
    let mock_response = json!({
        "success": true,
        "data": {
            "algo_order_id": "123456",
            "client_order_id": "my_stop_loss_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": "0.1",
            "trigger_price": "50000",
            "status": "CANCELLED",
            "reduce_only": true,
            "created_time": 1677721600000,
            "updated_time": 1677721700000
        }
    });

    let _m = server.mock("DELETE", "/v1/algo-order/PERP_BTC_USDC/by-client-order-id/my_stop_loss_1")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = Client::new(
        &server.url(),
        Some("test_key"),
        Some("test_secret"),
        Some("test_account"),
    ).unwrap();

    let result = client.cancel_algo_order_by_client_id("PERP_BTC_USDC", "my_stop_loss_1").await;
    assert!(result.is_ok());

    let order = result.unwrap();
    assert_eq!(order.client_order_id.unwrap(), "my_stop_loss_1");
    assert_eq!(order.status.to_string(), "CANCELLED");
}

#[tokio::test]
async fn test_get_algo_orders() {
    let mut server = mockito::Server::new();
    let mock_response = json!({
        "success": true,
        "data": {
            "rows": [
                {
                    "algo_order_id": "123456",
                    "client_order_id": "my_stop_loss_1",
                    "symbol": "PERP_BTC_USDC",
                    "order_type": "STOP_MARKET",
                    "side": "SELL",
                    "quantity": "0.1",
                    "trigger_price": "50000",
                    "status": "NEW",
                    "reduce_only": true,
                    "created_time": 1677721600000,
                    "updated_time": 1677721600000
                }
            ],
            "total": 1,
            "current_page": 1,
            "page_size": 10
        }
    });

    let _m = server.mock("GET", "/v1/algo-orders")
        .match_header("orderly-key", "test_key")
        .match_header("orderly-account-id", "test_account")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let client = Client::new(
        &server.url(),
        Some("test_key"),
        Some("test_secret"),
        Some("test_account"),
    ).unwrap();

    let result = client.get_algo_orders(GetAlgoOrdersParams::default()).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.rows.len(), 1);
    assert_eq!(response.total, 1);
    assert_eq!(response.current_page, 1);
    assert_eq!(response.page_size, 10);

    let order = &response.rows[0];
    assert_eq!(order.algo_order_id, "123456");
    assert_eq!(order.symbol, "PERP_BTC_USDC");
    assert_eq!(order.order_type, AlgoOrderType::StopMarket);
    assert_eq!(order.side, Side::Sell);
}

#[tokio::test]
async fn test_validation_errors() {
    let client = Client::new(
        "http://localhost",
        Some("test_key"),
        Some("test_secret"),
        Some("test_account"),
    ).unwrap();

    // Test empty symbol in create order
    let request = CreateAlgoOrderRequest {
        symbol: "".to_string(),
        order_type: AlgoOrderType::StopMarket,
        side: Side::Sell,
        quantity: dec!(0.1),
        trigger_price: dec!(50000),
        limit_price: None,
        trailing_delta: None,
        client_order_id: None,
        reduce_only: None,
    };
    let result = client.create_algo_order(request).await;
    assert!(matches!(result, Err(orderly_connector_rs::error::OrderlyError::ValidationError(_))));

    // Test empty symbol in cancel order
    let result = client.cancel_algo_order("", "123456").await;
    assert!(matches!(result, Err(orderly_connector_rs::error::OrderlyError::ValidationError(_))));

    // Test empty order ID in cancel order
    let result = client.cancel_algo_order("PERP_BTC_USDC", "").await;
    assert!(matches!(result, Err(orderly_connector_rs::error::OrderlyError::ValidationError(_))));

    // Test empty symbol in cancel by client ID
    let result = client.cancel_algo_order_by_client_id("", "my_order_1").await;
    assert!(matches!(result, Err(orderly_connector_rs::error::OrderlyError::ValidationError(_))));

    // Test empty client order ID in cancel by client ID
    let result = client.cancel_algo_order_by_client_id("PERP_BTC_USDC", "").await;
    assert!(matches!(result, Err(orderly_connector_rs::error::OrderlyError::ValidationError(_))));
} 