use mockito::{self, Server};
use orderly_connector_rs::{
    rest::{client::Credentials, OrderlyService},
    types::{AlgoOrderType, CreateAlgoOrderRequest, OrderStatus, Side},
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
async fn test_take_profit_order() {
    let mut server = Server::new_async().await;

    // Mock successful TP order response
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "algo_order_id": "tp_123",
            "client_order_id": "tp_btc_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "TAKE_PROFIT_LIMIT",
            "side": "SELL",
            "quantity": 0.01,
            "trigger_price": 52000.0,
            "limit_price": 51900.0,
            "trailing_delta": null,
            "status": "NEW",
            "reduce_only": true,
            "triggered_order_id": null,
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
        .match_body(mockito::Matcher::Json(json!({
            "symbol": "PERP_BTC_USDC",
            "order_type": "TAKE_PROFIT_LIMIT",
            "side": "SELL",
            "quantity": 0.01,
            "trigger_price": 52000.0,
            "limit_price": 51900.0,
            "client_order_id": "tp_btc_1",
            "reduce_only": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let tp_order = CreateAlgoOrderRequest {
        symbol: "PERP_BTC_USDC".to_string(),
        order_type: AlgoOrderType::TakeProfitLimit,
        side: Side::Sell,
        quantity: 0.01,
        trigger_price: 52000.0,
        limit_price: Some(51900.0),
        trailing_delta: None,
        client_order_id: Some("tp_btc_1".to_string()),
        reduce_only: Some(true),
    };

    let result = client.create_algo_order(&creds, tp_order).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.data.algo_order_id, 123);
    assert_eq!(response.data.status, OrderStatus::New);
}

#[tokio::test]
async fn test_stop_loss_order() {
    let mut server = Server::new_async().await;

    // Mock successful SL order response
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "algo_order_id": "sl_123",
            "client_order_id": "sl_btc_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": 0.01,
            "trigger_price": 48000.0,
            "limit_price": null,
            "trailing_delta": null,
            "status": "NEW",
            "reduce_only": true,
            "triggered_order_id": null,
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
        .match_body(mockito::Matcher::Json(json!({
            "symbol": "PERP_BTC_USDC",
            "order_type": "STOP_MARKET",
            "side": "SELL",
            "quantity": 0.01,
            "trigger_price": 48000.0,
            "client_order_id": "sl_btc_1",
            "reduce_only": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let sl_order = CreateAlgoOrderRequest {
        symbol: "PERP_BTC_USDC".to_string(),
        order_type: AlgoOrderType::StopMarket,
        side: Side::Sell,
        quantity: 0.01,
        trigger_price: 48000.0,
        limit_price: None,
        trailing_delta: None,
        client_order_id: Some("sl_btc_1".to_string()),
        reduce_only: Some(true),
    };

    let result = client.create_algo_order(&creds, sl_order).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.data.algo_order_id, 123);
    assert_eq!(response.data.status, OrderStatus::New);
}

#[tokio::test]
async fn test_trailing_stop_order() {
    let mut server = Server::new_async().await;

    // Mock successful trailing stop order response
    let mock_response = json!({
        "success": true,
        "timestamp": 1677721600123_u64,
        "data": {
            "algo_order_id": "ts_123",
            "client_order_id": "trailing_stop_1",
            "symbol": "PERP_BTC_USDC",
            "order_type": "TRAILING_STOP",
            "side": "SELL",
            "quantity": 0.01,
            "trigger_price": 47500.0,
            "limit_price": null,
            "trailing_delta": 500.0,
            "status": "NEW",
            "reduce_only": true,
            "triggered_order_id": null,
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
        .match_body(mockito::Matcher::Json(json!({
            "symbol": "PERP_BTC_USDC",
            "order_type": "TRAILING_STOP",
            "side": "SELL",
            "quantity": 0.01,
            "trigger_price": 47500.0,
            "trailing_delta": 500.0,
            "client_order_id": "trailing_stop_1",
            "reduce_only": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = OrderlyService::with_base_url(&server.url(), None).unwrap();
    let creds = test_credentials();

    let trailing_stop = CreateAlgoOrderRequest {
        symbol: "PERP_BTC_USDC".to_string(),
        order_type: AlgoOrderType::TrailingStop,
        side: Side::Sell,
        quantity: 0.01,
        trigger_price: 47500.0,
        limit_price: None,
        trailing_delta: Some(500.0),
        client_order_id: Some("trailing_stop_1".to_string()),
        reduce_only: Some(true),
    };

    let result = client.create_algo_order(&creds, trailing_stop).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.data.algo_order_id, 123);
    assert_eq!(response.data.status, OrderStatus::New);
}
