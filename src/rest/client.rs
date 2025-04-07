use crate::auth::{self, get_timestamp_ms};
use crate::error::{OrderlyError, Result};
use crate::types::*;
use log::warn;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client as HttpClient, Method, Request, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use url::Url;

const MAINNET_API_URL: &str = "https://api-evm.orderly.network";
const TESTNET_API_URL: &str = "https://testnet-api-evm.orderly.network";
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

/// A client for interacting with the Orderly Network REST API.
///
/// This client provides methods for both public and private endpoints,
/// handling authentication, request signing, and response parsing.
///
/// # Examples
///
/// ```no_run
/// use orderly_connector_rs::rest::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new(
///         "your_api_key".to_string(),
///         "your_secret".to_string(),
///         "your_account_id".to_string(),
///         true, // is_testnet
///         None, // timeout_sec
///     ).expect("Failed to create client");
///
///     // Get system status
///     let status = client.get_system_status().await.expect("Failed to get status");
///     println!("System status: {:?}", status);
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    /// The underlying HTTP client used for making requests.
    http_client: HttpClient,
    /// The base URL for the Orderly API (either mainnet or testnet).
    base_url: Url,
    /// The public API key provided by Orderly Network.
    orderly_key: String,
    /// The private API secret provided by Orderly Network, used for signing requests.
    orderly_secret: String,
    /// The user's unique account identifier on Orderly Network.
    orderly_account_id: String,
    // timeout is configured directly in the HttpClient
}

impl Client {
    /// Creates a new Orderly REST API client.
    ///
    /// # Arguments
    ///
    /// * `orderly_key` - Your Orderly API key public key
    /// * `orderly_secret` - Your Orderly API secret - private key
    /// * `orderly_account_id` - Your Orderly account ID
    /// * `is_testnet` - Whether to use testnet (true) or mainnet (false)
    /// * `timeout_sec` - Optional timeout in seconds for HTTP requests
    ///
    /// # Returns
    ///
    /// A `Result` containing the configured client or an error if initialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orderly_connector_rs::rest::Client;
    ///
    /// let client = Client::new(
    ///     "your_api_key".to_string(),
    ///     "your_secret".to_string(),
    ///     "your_account_id".to_string(),
    ///     true, // is_testnet
    ///     Some(30), // timeout_sec
    /// ).expect("Failed to create client");
    /// ```
    pub fn new(
        orderly_key: String,
        orderly_secret: String,
        orderly_account_id: String,
        is_testnet: bool,
        timeout_sec: Option<u64>,
    ) -> Result<Self> {
        let base_url_str = if is_testnet {
            TESTNET_API_URL
        } else {
            MAINNET_API_URL
        };
        let base_url = Url::parse(base_url_str)?;

        let timeout_duration = Duration::from_secs(timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SECONDS));

        let http_client = HttpClient::builder().timeout(timeout_duration).build()?; // Propagates reqwest::Error via From trait in OrderlyError

        Ok(Self {
            http_client,
            base_url,
            orderly_key,
            orderly_secret,
            orderly_account_id,
        })
    }

    /// Builds a signed reqwest::Request.
    async fn build_signed_request<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<T>,
    ) -> Result<Request> {
        let timestamp = get_timestamp_ms()?;
        let full_url = self.base_url.join(path)?;

        let body_str = match &body {
            Some(b) => serde_json::to_string(b)?, // Propagates SerdeError
            None => String::new(),
        };

        let message_to_sign = format!("{}{}{}{}", timestamp, method.as_str(), path, body_str);
        let signature = auth::generate_signature(&self.orderly_secret, &message_to_sign)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("orderly-timestamp"),
            HeaderValue::from(timestamp),
        );
        headers.insert(
            HeaderName::from_static("orderly-key"),
            HeaderValue::from_str(&self.orderly_key)?,
        );
        headers.insert(
            HeaderName::from_static("orderly-signature"),
            HeaderValue::from_str(&signature)?,
        );
        headers.insert(
            HeaderName::from_static("orderly-account-id"),
            HeaderValue::from_str(&self.orderly_account_id)?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let mut request_builder = self.http_client.request(method, full_url).headers(headers);

        if let Some(b) = body {
            request_builder = request_builder.json(&b);
        }

        Ok(request_builder.build()?)
    }

    /// Sends a request and handles the response, parsing success or error.
    async fn send_request<T: DeserializeOwned>(&self, request: Request) -> Result<T> {
        let response = self.http_client.execute(request).await?;
        let status = response.status();
        let headers = response.headers().clone();

        if status.is_success() {
            // Attempt to parse successful response
            let parsed_body = response.json::<T>().await?;
            Ok(parsed_body)
        } else {
            // Attempt to parse error response body
            let error_body: Value = match response.json::<Value>().await {
                Ok(val) => val,
                Err(_) => {
                    // If parsing error body fails, create a generic error
                    let error_kind = if status.is_client_error() {
                        OrderlyError::ClientError {
                            status,
                            code: 0, // Unknown code
                            message: format!(
                                "Request failed with status {} (could not parse error body)",
                                status
                            ),
                            data: None,
                            header: headers,
                        }
                    } else {
                        OrderlyError::ServerError {
                            status,
                            code: 0, // Unknown code
                            message: format!(
                                "Request failed with status {} (could not parse error body)",
                                status
                            ),
                            header: headers,
                        }
                    };
                    return Err(error_kind);
                }
            };

            let code = error_body["code"].as_i64().unwrap_or(0);
            let message = error_body["message"]
                .as_str()
                .unwrap_or("Unknown error message")
                .to_string();
            let data = error_body.get("data").cloned(); // Optional 'data' field in errors

            let error = if status.is_client_error() {
                OrderlyError::ClientError {
                    status,
                    code,
                    message,
                    data,
                    header: headers,
                }
            } else {
                OrderlyError::ServerError {
                    status,
                    code,
                    message,
                    header: headers,
                }
            };
            Err(error)
        }
    }

    /// Sends an unsigned public request and handles the response.
    async fn send_public_request<T: DeserializeOwned>(&self, request: Request) -> Result<T> {
        let response = self.http_client.execute(request).await?;
        Self::handle_response(response).await // Use a shared response handler
    }

    /// Shared logic to handle response status and body parsing (for both public and private).
    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T> {
        let status = response.status();
        let headers = response.headers().clone();

        if status.is_success() {
            let parsed_body = response.json::<T>().await?;
            Ok(parsed_body)
        } else {
            let error_body: Value = match response.json::<Value>().await {
                Ok(val) => val,
                Err(_) => {
                    let error_kind = if status.is_client_error() {
                        OrderlyError::ClientError {
                            status,
                            code: 0,
                            message: format!(
                                "Request failed with status {} (could not parse error body)",
                                status
                            ),
                            data: None,
                            header: headers,
                        }
                    } else {
                        OrderlyError::ServerError {
                            status,
                            code: 0,
                            message: format!(
                                "Request failed with status {} (could not parse error body)",
                                status
                            ),
                            header: headers,
                        }
                    };
                    return Err(error_kind);
                }
            };

            let code = error_body["code"].as_i64().unwrap_or(0);
            let message = error_body["message"]
                .as_str()
                .unwrap_or("Unknown error message")
                .to_string();
            let data = error_body.get("data").cloned();

            let error = if status.is_client_error() {
                OrderlyError::ClientError {
                    status,
                    code,
                    message,
                    data,
                    header: headers,
                }
            } else {
                OrderlyError::ServerError {
                    status,
                    code,
                    message,
                    header: headers,
                }
            };
            Err(error)
        }
    }

    // --- Public Endpoints ---

    /// Retrieves the current system status and maintenance information.
    ///
    /// This endpoint provides information about the current state of the Orderly Network,
    /// including maintenance windows and system status.
    ///
    /// # Returns
    ///
    /// A `Result` containing the system status information as a JSON `Value` or an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::Client;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = Client::new("key".to_string(), "secret".to_string(), "account".to_string(), true, None).unwrap();
    /// let status = client.get_system_status().await.expect("Failed to get status");
    /// println!("System status: {:?}", status);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/public/get-system-status
    pub async fn get_system_status(&self) -> Result<Value> {
        let path = "/v1/public/system_info";
        let url = self.base_url.join(path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Retrieves exchange information, optionally filtered by symbol.
    ///
    /// This endpoint provides detailed information about available trading pairs,
    /// including trading rules, fees, and other relevant details.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional symbol to filter the exchange information
    ///
    /// # Returns
    ///
    /// A `Result` containing the exchange information as a JSON `Value` or an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::Client;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = Client::new("key".to_string(), "secret".to_string(), "account".to_string(), true, None).unwrap();
    /// // Get info for all symbols
    /// let all_info = client.get_exchange_info(None).await.expect("Failed to get info");
    ///
    /// // Get info for a specific symbol
    /// let symbol_info = client.get_exchange_info(Some("PERP_ETH_USDC")).await.expect("Failed to get symbol info");
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/public/get-exchange-info
    pub async fn get_exchange_info(&self, symbol: Option<&str>) -> Result<Value> {
        let path = match symbol {
            Some(s) => format!("/v1/public/info/{}", s),
            None => "/v1/public/info".to_string(),
        };
        let url = self.base_url.join(&path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Retrieves futures contract information, optionally filtered by symbol.
    /// Corresponds to GET /v1/public/futures and GET /v1/public/futures/{symbol}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/public/get-futures-info
    pub async fn get_futures_info(&self, symbol: Option<&str>) -> Result<Value> {
        let path = match symbol {
            Some(s) => format!("/v1/public/futures/{}", s),
            None => "/v1/public/futures".to_string(),
        };
        let url = self.base_url.join(&path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    // --- Private Endpoints (Orders) ---

    /// Creates a new order.
    /// Corresponds to POST /v1/order
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/create-order
    pub async fn create_order(
        &self,
        order_req: CreateOrderRequest<'_>,
    ) -> Result<CreateOrderResponse> {
        let request = self
            .build_signed_request(Method::POST, "/v1/order", Some(order_req))
            .await?;
        self.send_request::<CreateOrderResponse>(request).await
    }

    /// Retrieves a specific order by its ID.
    /// Corresponds to GET /v1/order/{order_id}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-order
    pub async fn get_order(&self, order_id: u64) -> Result<GetOrderResponse> {
        let path = format!("/v1/order/{}", order_id);
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetOrderResponse>(request).await
    }

    /// Cancels an existing order by its ID.
    /// Corresponds to DELETE /v1/order?order_id={order_id}&symbol={symbol}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/cancel-order
    pub async fn cancel_order(&self, order_id: u64, symbol: &str) -> Result<CancelOrderResponse> {
        let path = format!("/v1/order?order_id={}&symbol={}", order_id, symbol);
        let request = self
            .build_signed_request::<()>(Method::DELETE, &path, None)
            .await?;
        self.send_request::<CancelOrderResponse>(request).await
    }

    /// Retrieves multiple orders based on filter parameters.
    /// Corresponds to GET /v1/orders
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-orders
    pub async fn get_orders(
        &self,
        params: Option<GetOrdersParams<'_>>,
    ) -> Result<GetOrdersResponse> {
        let mut path = "/v1/orders".to_string();
        if let Some(p) = params {
            // TODO: Implement proper query string building from GetOrdersParams
            // For now, assumes no params or manual construction if needed
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                // Handle serialization error if necessary
                warn!("Failed to serialize GetOrdersParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetOrdersResponse>(request).await
    }

    // ===== Account Information =====

    /// Get current account information.
    /// GET /v1/client/info
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-account-info
    pub async fn get_account_info(&self) -> Result<GetAccountInfoResponse> {
        let request = self
            .build_signed_request::<()>(Method::GET, "/v1/client/info", None)
            .await?;
        self.send_request::<GetAccountInfoResponse>(request).await
    }

    // ===== Holdings / Balances =====

    /// Get current holdings (balances) for all tokens.
    /// GET /v1/client/holding
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-holding
    pub async fn get_holding(&self) -> Result<GetHoldingResponse> {
        let request = self
            .build_signed_request::<()>(Method::GET, "/v1/client/holding", None)
            .await?;
        self.send_request::<GetHoldingResponse>(request).await
    }

    // ===== Positions =====

    /// Get all current positions.
    /// GET /v1/positions
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-all-positions-info
    pub async fn get_positions(&self) -> Result<GetPositionsResponse> {
        let request = self
            .build_signed_request::<()>(Method::GET, "/v1/positions", None)
            .await?;
        self.send_request::<GetPositionsResponse>(request).await
    }

    /// Get position for a specific symbol.
    /// GET /v1/position/{symbol}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-one-position-info
    pub async fn get_position(&self, symbol: &str) -> Result<GetSinglePositionResponse> {
        let path = format!("/v1/position/{}", symbol);
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetSinglePositionResponse>(request)
            .await
    }

    // ===== Asset History (Deposits/Withdrawals) =====

    /// Get asset history (deposits, withdrawals).
    /// GET /v1/asset/history
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-asset-history
    pub async fn get_asset_history(
        &self,
        params: Option<GetAssetHistoryParams<'_>>,
    ) -> Result<GetAssetHistoryResponse> {
        let mut path = "/v1/asset/history".to_string();
        if let Some(p) = params {
            // TODO: Implement proper query string building from GetAssetHistoryParams
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetAssetHistoryParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetAssetHistoryResponse>(request).await
    }

    // ===== Trades =====

    /// Get trade history.
    /// GET /v1/trades
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-trades
    pub async fn get_trades(
        &self,
        params: Option<GetTradesParams<'_>>,
    ) -> Result<GetTradesResponse> {
        let mut path = "/v1/trades".to_string();
        if let Some(p) = params {
            // TODO: Implement proper query string building from GetTradesParams
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetTradesParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetTradesResponse>(request).await
    }

    /// Get specific trade by ID.
    /// GET /v1/trade/{trade_id}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-trade
    pub async fn get_trade(&self, trade_id: u64) -> Result<Value> {
        let path = format!("/v1/trade/{}", trade_id);
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<Value>(request).await
    }

    // ===== Client Statistics =====

    /// Get client statistics (e.g., 30d volume, VIP tier).
    /// GET /v1/client/statistics
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-client-statistics
    pub async fn get_client_statistics(&self) -> Result<GetClientStatisticsResponse> {
        let request = self
            .build_signed_request::<()>(Method::GET, "/v1/client/statistics", None)
            .await?;
        self.send_request::<GetClientStatisticsResponse>(request)
            .await
    }

    // TODO: Implement endpoints for Liquidations, Fees, Settlement, Referrals, Broker, Delegate Signer, IP Restrictions etc.

    // ===== Withdrawals =====

    /// Request a withdrawal.
    /// POST /v1/withdraw_request
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/request-withdrawal
    pub async fn request_withdrawal(
        &self,
        withdraw_req: WithdrawRequest<'_>,
    ) -> Result<WithdrawResponse> {
        let request = self
            .build_signed_request(Method::POST, "/v1/withdraw_request", Some(withdraw_req))
            .await?;
        self.send_request::<WithdrawResponse>(request).await
    }
    // Note: Withdrawal history fetched via get_asset_history

    // ===== Fee Rates =====

    /// Get current fee rates for the user.
    /// GET /v1/client/fee_rates
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-fee-rates
    pub async fn get_fee_rates(&self) -> Result<GetFeeRatesResponse> {
        let request = self
            .build_signed_request::<()>(Method::GET, "/v1/client/fee_rates", None)
            .await?;
        self.send_request::<GetFeeRatesResponse>(request).await
    }

    // ===== Liquidations =====

    /// Get liquidation history for the user's positions.
    /// GET /v1/liquidations
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-liquidations
    pub async fn get_liquidations(
        &self,
        params: Option<GetLiquidationsParams<'_>>,
    ) -> Result<GetLiquidationsResponse> {
        let mut path = "/v1/liquidations".to_string();
        if let Some(p) = params {
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetLiquidationsParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetLiquidationsResponse>(request).await
    }

    // ===== PnL Settlement =====

    /// Get PnL settlement history.
    /// GET /v1/settlements
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-settlement-history
    pub async fn get_settlement_history(
        &self,
        params: Option<GetSettlementsParams<'_>>,
    ) -> Result<GetSettlementsResponse> {
        let mut path = "/v1/settlements".to_string();
        if let Some(p) = params {
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetSettlementsParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetSettlementsResponse>(request).await
    }

    // ===== Funding Fee =====

    /// Get funding fee history.
    /// GET /v1/funding_fee/history
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-funding-fee-history
    pub async fn get_funding_fee_history(
        &self,
        symbol: &str,
        params: Option<GetFundingFeeParams>,
    ) -> Result<GetFundingFeeHistoryResponse> {
        let mut path = format!("/v1/funding_fee/history?symbol={}", symbol);
        if let Some(p) = params {
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('&');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetFundingFeeParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        self.send_request::<GetFundingFeeHistoryResponse>(request)
            .await
    }

    // // ===== Algo Orders =====

    // /// Create an algorithmic order (e.g., stop order).
    // /// POST /v1/algo/order
    // ///
    // /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/create-algo-order
    // pub async fn create_algo_order(
    //     &self,
    //     algo_order_req: CreateAlgoOrderRequest<'_>,
    // ) -> Result<Value> {
    //     let request = self
    //         .build_signed_request(Method::POST, "/v1/algo/order", Some(algo_order_req))
    //         .await?;
    //     self.send_request::<Value>(request).await
    // }
}
