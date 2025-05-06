use env_logger;
use mockito;
use orderly_connector_rs::{
    rest::{client::Credentials, OrderlyService},
    types::{AlgoOrderType, CreateAlgoOrderRequest, GetAlgoOrdersParams, OrderStatus, Side},
};
use serde_json::json;

fn setup_logger() {
    // Helper function for logger init
    let _ = env_logger::builder().is_test(true).try_init();
}

#[tokio::test]
async fn test_create_algo_order() {
    setup_logger(); // Init logger
    let mut server = mockito::Server::new_async().await;
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "algo_order_id": "123456",
            "client_order_id": "my_stop_loss_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": 0.1,
            "trigger_price": 50000.0,
            "status": "NEW",
            "reduce_only": true,
            "created_time": 1677721600000_i64,
            "updated_time": 1677721600000_i64
        }
    });

    let _m = server
        .mock("POST", "/v1/algo-order")
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
    let creds = Credentials {
        orderly_key: "test_key",
        orderly_secret: "11111111111111111111111111111111",
        orderly_account_id: "test_account",
    };

    let request = CreateAlgoOrderRequest {
        symbol: "PERP_BTC_USDC".to_string(),
        order_type: AlgoOrderType::StopMarket,
        side: Side::Sell,
        quantity: 0.1,
        trigger_price: 50000.0,
        limit_price: None,
        trailing_delta: None,
        client_order_id: Some("my_stop_loss_1".to_string()),
        reduce_only: Some(true),
    };

    let result = client.create_algo_order(&creds, request).await;
    if let Err(e) = &result {
        println!("test_create_algo_order error: {:?}", e);
    }
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.success);
    let order = response.data;

    assert_eq!(order.algo_order_id, 123456);
    assert_eq!(order.symbol, "PERP_BTC_USDC");
    assert_eq!(order.order_type, AlgoOrderType::StopMarket);
    assert_eq!(order.side, Side::Sell);
}

#[tokio::test]
async fn test_cancel_algo_order() {
    setup_logger(); // Init logger
    let mut server = mockito::Server::new_async().await;
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721700123_u64,
        "data": {
            "algo_order_id": "123456",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": 0.1,
            "trigger_price": 50000.0,
            "status": "CANCELLED",
            "reduce_only": true,
            "created_time": 1677721600000_i64,
            "updated_time": 1677721700000_i64
        }
    });

    let _m = server
        .mock("DELETE", "/v1/algo-order/PERP_BTC_USDC/123456")
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
    let creds = Credentials {
        orderly_key: "test_key",
        orderly_secret: "11111111111111111111111111111111",
        orderly_account_id: "test_account",
    };

    let result = client
        .cancel_algo_order(&creds, "PERP_BTC_USDC", "123456")
        .await;
    if let Err(e) = &result {
        println!("test_cancel_algo_order error: {:?}", e);
    }
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.success);
    let order = response.data;

    assert_eq!(order.algo_order_id, 123456);
    assert_eq!(order.algo_status.unwrap(), OrderStatus::Cancelled);
}

#[tokio::test]
async fn test_cancel_algo_order_by_client_id() {
    setup_logger(); // Init logger
    let mut server = mockito::Server::new_async().await;
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721700321_u64,
        "data": {
            "algo_order_id": "123456",
            "client_order_id": "my_stop_loss_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": 0.1,
            "trigger_price": 50000.0,
            "status": "CANCELLED",
            "reduce_only": true,
            "created_time": 1677721600000_i64,
            "updated_time": 1677721700000_i64
        }
    });

    let _m = server
        .mock(
            "DELETE",
            "/v1/algo-order/PERP_BTC_USDC/by-client-order-id/my_stop_loss_1",
        )
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
    let creds = Credentials {
        orderly_key: "test_key",
        orderly_secret: "11111111111111111111111111111111",
        orderly_account_id: "test_account",
    };

    let result = client
        .cancel_algo_order_by_client_id(&creds, "PERP_BTC_USDC", "my_stop_loss_1")
        .await;
    if let Err(e) = &result {
        println!("test_cancel_algo_order_by_client_id error: {:?}", e);
    }
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.success);
    let order = response.data;

    assert_eq!(order.client_order_id.unwrap(), "my_stop_loss_1");
    assert_eq!(order.algo_status.unwrap(), OrderStatus::Cancelled);
}

#[tokio::test]
async fn test_get_algo_orders() {
    setup_logger(); // Init logger
    let mut server = mockito::Server::new_async().await;
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721800456_u64,
        "data": {
            "rows": [
                {
                    "algo_order_id": "123456",
                    "client_order_id": "my_stop_loss_1",
                    "symbol": "PERP_BTC_USDC",
                    "order_type": "STOP_MARKET",
                    "side": "SELL",
                    "quantity": 0.1,
                    "trigger_price": 50000.0,
                    "status": "NEW",
                    "reduce_only": true,
                    "created_time": 1677721600000_i64,
                    "updated_time": 1677721600000_i64
                }
            ],
            "total": 1,
            "current_page": 1,
            "page_size": 10
        }
    });

    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/v1/algo-orders(\?.*)?$".to_string()),
        )
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
    let creds = Credentials {
        orderly_key: "test_key",
        orderly_secret: "11111111111111111111111111111111",
        orderly_account_id: "test_account",
    };

    let result = client
        .get_algo_orders(&creds, GetAlgoOrdersParams::default())
        .await;
    if let Err(e) = &result {
        println!("test_get_algo_orders error: {:?}", e);
    }
    assert!(result.is_ok());

    let response = result.unwrap();
    let orderly_connector_rs::types::SuccessResponse { success, data, .. } = response;
    assert!(success);
    let rows = &data.rows;
    let meta = &data.meta;

    assert_eq!(rows.len(), 1);
    assert_eq!(meta.total, 1);
    assert_eq!(meta.current_page, 1);
    assert_eq!(meta.records_per_page, 10);

    let order = &rows[0];
    assert_eq!(order.algo_order_id, 123456);
    assert_eq!(order.symbol, "PERP_BTC_USDC");
    assert_eq!(order.order_type, AlgoOrderType::StopMarket);
    assert_eq!(order.side, Side::Sell);
}

#[tokio::test]
async fn test_validation_errors() {
    setup_logger(); // Init logger
    let client = OrderlyService::new(true, None).unwrap();
    let creds = Credentials {
        orderly_key: "test_key",
        orderly_secret: "11111111111111111111111111111111",
        orderly_account_id: "test_account",
    };

    // Test empty symbol in create order
    let request = CreateAlgoOrderRequest {
        symbol: "".to_string(),
        order_type: AlgoOrderType::StopMarket,
        side: Side::Sell,
        quantity: 0.1,
        trigger_price: 50000.0,
        limit_price: None,
        trailing_delta: None,
        client_order_id: None,
        reduce_only: None,
    };
    let result = client.create_algo_order(&creds, request).await;
    assert!(matches!(
        result,
        Err(orderly_connector_rs::error::OrderlyError::ValidationError(
            _
        ))
    ));

    // Test empty symbol in cancel order
    let result = client.cancel_algo_order(&creds, "", "123456").await;
    assert!(matches!(
        result,
        Err(orderly_connector_rs::error::OrderlyError::ValidationError(
            _
        ))
    ));

    // Test empty order ID in cancel order
    let result = client.cancel_algo_order(&creds, "PERP_BTC_USDC", "").await;
    assert!(matches!(
        result,
        Err(orderly_connector_rs::error::OrderlyError::ValidationError(
            _
        ))
    ));

    // Test empty symbol in cancel by client ID
    let result = client
        .cancel_algo_order_by_client_id(&creds, "", "my_order_1")
        .await;
    assert!(matches!(
        result,
        Err(orderly_connector_rs::error::OrderlyError::ValidationError(
            _
        ))
    ));

    // Test empty client order ID in cancel by client ID
    let result = client
        .cancel_algo_order_by_client_id(&creds, "PERP_BTC_USDC", "")
        .await;
    assert!(matches!(
        result,
        Err(orderly_connector_rs::error::OrderlyError::ValidationError(
            _
        ))
    ));
}
