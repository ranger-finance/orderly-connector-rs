use crate::auth::get_timestamp_ms;
use crate::error::{OrderlyError, Result};
use crate::{auth, types::*};
use log::{error, warn};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client as HttpClient, Method, Request, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use url::Url;

const MAINNET_API_URL: &str = "https://api-evm.orderly.network";
const TESTNET_API_URL: &str = "https://testnet-api-evm.orderly.network";
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

/// Holds the necessary credentials for authenticating with private Orderly endpoints.
#[derive(Debug, Clone)] // Clone is useful, Debug for logging
pub struct Credentials<'a> {
    /// The public API key provided by Orderly Network.
    pub orderly_key: &'a str,
    /// The private API secret provided by Orderly Network, used for signing requests.
    pub orderly_secret: &'a str,
    /// The user's unique account identifier on Orderly Network.
    pub orderly_account_id: &'a str,
}

/// A service client for interacting with the Orderly Network REST API.
///
/// This service holds shared components like the HTTP client and base URL,
/// allowing multiple users' requests to be handled efficiently.
/// User-specific credentials should be passed into the methods requiring authentication.
///
/// # Examples
///
/// ```no_run
/// use orderly_connector_rs::rest::OrderlyService;
/// use std::sync::Arc;
/// use orderly_connector_rs::error::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Optional: Load credentials from .env file
///     // dotenv::dotenv().ok();
///     // let api_key = std::env::var("ORDERLY_API_KEY").expect("ORDERLY_API_KEY not set");
///     // let secret = std::env::var("ORDERLY_SECRET").expect("ORDERLY_SECRET not set");
///     // let account_id = std::env::var("ORDERLY_ACCOUNT_ID").expect("ORDERLY_ACCOUNT_ID not set");
///     // Example credentials (replace with actual values or load from env)
///     // use orderly_connector_rs::rest::client::Credentials;
///     // let creds = Credentials {
///     //     orderly_key: api_key,
///     //     orderly_secret: secret,
///     //     orderly_account_id: account_id,
///     // };
///
///     // Create a new service for the testnet, no timeout
///     let service = OrderlyService::new(true, None)?;
///
///     // Example: Get system status (public endpoint, no credentials needed)
///     let status = service.get_system_status().await?;
///     println!("System Status: {:?}", status);
///
///     // To call private endpoints, ensure credentials are loaded (e.g., via .env)
///     // and use methods that require authentication, like get_account_info.
///     // let account_info = service.get_account_info().await?;
///     // println!("Account Info: {:?}", account_info);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct OrderlyService {
    /// The underlying HTTP client used for making requests.
    http_client: HttpClient,
    /// The base URL for the Orderly API (either mainnet or testnet).
    base_url: Url,
    // User-specific fields removed
    // timeout is configured directly in the HttpClient
}

impl OrderlyService {
    /// Creates a new Orderly REST API service.
    ///
    /// # Arguments
    ///
    /// * `is_testnet` - Whether to use testnet (true) or mainnet (false)
    /// * `timeout_sec` - Optional timeout in seconds for HTTP requests
    ///
    /// # Returns
    ///
    /// A `Result` containing the configured service or an error if initialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orderly_connector_rs::rest::OrderlyService;
    ///
    /// let service = OrderlyService::new(
    ///     true, // is_testnet
    ///     Some(30), // timeout_sec
    /// ).expect("Failed to create service");
    /// ```
    pub fn new(is_testnet: bool, timeout_sec: Option<u64>) -> Result<Self> {
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
        })
    }

    /// Creates a new Orderly REST API service with a custom base URL.
    ///
    /// This is useful for testing environments where a mock server is used.
    ///
    /// # Arguments
    ///
    /// * `base_url_str` - The base URL to use for the API.
    /// * `timeout_sec` - Optional timeout in seconds for HTTP requests
    ///
    /// # Returns
    ///
    /// A `Result` containing the configured service or an error if initialization fails.
    pub fn with_base_url(base_url_str: &str, timeout_sec: Option<u64>) -> Result<Self> {
        let base_url = Url::parse(base_url_str)?;
        let timeout_duration = Duration::from_secs(timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SECONDS));
        let http_client = HttpClient::builder().timeout(timeout_duration).build()?;
        Ok(Self {
            http_client,
            base_url,
        })
    }

    /// Builds a signed reqwest::Request using provided credentials.
    async fn build_signed_request<T: Serialize>(
        &self,
        creds: &Credentials<'_>, // Accept credentials
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
        // Use credentials passed in
        let signature = auth::generate_signature(creds.orderly_secret, &message_to_sign)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("orderly-timestamp"),
            HeaderValue::from(timestamp),
        );
        // Use credentials passed in
        headers.insert(
            HeaderName::from_static("orderly-key"),
            HeaderValue::from_str(creds.orderly_key)?,
        );
        headers.insert(
            HeaderName::from_static("orderly-signature"),
            HeaderValue::from_str(&signature)?,
        );
        // Use credentials passed in
        headers.insert(
            HeaderName::from_static("orderly-account-id"),
            HeaderValue::from_str(creds.orderly_account_id)?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let mut request_builder = self.http_client.request(method, full_url).headers(headers);

        if let Some(b) = body {
            request_builder = request_builder.json(&b);
        }

        Ok(request_builder.build()?) // Propagates reqwest::Error
    }

    /// Sends a request and handles the response, parsing success or error.
    async fn send_request<T: DeserializeOwned>(&self, request: Request) -> Result<T> {
        let response = self.http_client.execute(request).await?;
        let status = response.status();
        let headers = response.headers().clone();

        if status.is_success() {
            // Read the body text first
            let body_text = response.text().await?; // Consume response here

            // Attempt to parse the text as JSON
            match serde_json::from_str::<T>(&body_text) {
                // Parse from text
                Ok(parsed_body) => Ok(parsed_body),
                Err(e) => {
                    // Log the raw body text if JSON parsing fails
                    error!(
                        "Failed to parse successful response body (Status: {}). Error: {}. Body: {}",
                        status,
                        e,
                        body_text // We already have the text
                    );
                    // Determine if the serde error should be mapped to a specific OrderlyError
                    // For now, just wrap the serde error.
                    Err(OrderlyError::Serde(e)) // Wrap serde error
                }
            }
        } else {
            // Attempt to parse error response body
            let error_body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<Could not read error response body as text>".to_string());
            let error_body: Value = match serde_json::from_str(&error_body_text) {
                Ok(val) => val,
                Err(_) => {
                    // If parsing error body fails, create a generic error
                    let error_kind =
                        if status.is_client_error() {
                            OrderlyError::ClientError {
                                status,
                                code: 0, // Unknown code
                                message: format!(
                                "Request failed with status {} (could not parse error body: {})",
                                status, error_body_text // Include raw text in message
                            ),
                                data: None,
                                header: headers,
                            }
                        } else {
                            OrderlyError::ServerError {
                                status,
                                code: 0, // Unknown code
                                message: format!(
                                "Request failed with status {} (could not parse error body: {})",
                                status, error_body_text // Include raw text in message
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
                .unwrap_or(&error_body_text) // Use raw text as fallback message
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
            let error_body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<Could not read error response body as text>".to_string());
            let error_body: Value = match serde_json::from_str(&error_body_text) {
                Ok(val) => val,
                Err(_) => {
                    let error_kind =
                        if status.is_client_error() {
                            OrderlyError::ClientError {
                                status,
                                code: 0,
                                message: format!(
                                "Request failed with status {} (could not parse error body: {})",
                                status, error_body_text // Include raw text in message
                            ),
                                data: None,
                                header: headers,
                            }
                        } else {
                            OrderlyError::ServerError {
                                status,
                                code: 0,
                                message: format!(
                                "Request failed with status {} (could not parse error body: {})",
                                status, error_body_text // Include raw text in message
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
                .unwrap_or(&error_body_text) // Use raw text as fallback message
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
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// let status = service.get_system_status().await.expect("Failed to get status");
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
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// // Get info for all symbols
    /// let all_info = service.get_exchange_info(None).await.expect("Failed to get info");
    ///
    /// // Get info for a specific symbol
    /// let symbol_info = service.get_exchange_info(Some("PERP_ETH_USDC")).await.expect("Failed to get symbol info");
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/public/get-exchange-info
    pub async fn get_exchange_info(&self, symbol: Option<&str>) -> Result<ExchangeInfoResponse> {
        let path = match symbol {
            Some(s) => format!("/v1/public/info/{}", s),
            None => "/v1/public/info".to_string(),
        };
        let url = self.base_url.join(&path)?;
        let request = self.http_client.get(url).build()?;
        self.send_request::<ExchangeInfoResponse>(request).await
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

    /// Get funding rate history for all markets.
    /// GET /v1/public/market_info/funding_history
    ///
    /// This endpoint provides funding rate history information for all markets, including
    /// rates for different time periods (last, 1d, 3d, 7d, 14d, 30d, 90d, 180d).
    ///
    /// # Returns
    ///
    /// A `Result` containing the funding rate history information for all markets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// let funding_rates = service.get_funding_rate_history().await.expect("Failed to get funding rates");
    /// println!("Funding rates: {:?}", funding_rates);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/public/get-funding-rate-for-all-markets
    pub async fn get_funding_rate_history(&self) -> Result<GetFundingRateHistoryResponse> {
        let path = "/v1/public/market_info/funding_history";
        let url = self.base_url.join(path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Get open interest for all trading pairs.
    /// GET /v1/public/market_info/traders_open_interests
    ///
    /// This endpoint provides open interest information for all trading pairs,
    /// showing the total long and short positions held by traders.
    ///
    /// # Returns
    ///
    /// A `Result` containing the open interest information for all trading pairs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// let open_interests = service.get_open_interest().await.expect("Failed to get open interests");
    /// println!("Open interests: {:?}", open_interests);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/public/get-open-interests-for-all-symbols
    pub async fn get_open_interest(&self) -> Result<GetOpenInterestResponse> {
        let path = "/v1/public/market_info/traders_open_interests";
        let url = self.base_url.join(path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Get price changes for all trading pairs.
    /// GET /v1/public/market_info/price_changes
    ///
    /// This endpoint provides price information for all trading pairs at different time intervals:
    /// 5m, 30m, 1h, 4h, 24h, 3d, 7d, and 30d ago.
    ///
    /// # Returns
    ///
    /// A `Result` containing the price changes information for all trading pairs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// let price_changes = service.get_price_changes().await.expect("Failed to get price changes");
    /// println!("Price changes: {:?}", price_changes);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/public/get-price-info-for-all-symbols
    pub async fn get_price_changes(&self) -> Result<GetPriceChangesResponse> {
        let path = "/v1/public/market_info/price_changes";
        let url = self.base_url.join(path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Retrieves positions currently under liquidation.
    ///
    /// Corresponds to GET /v1/public/liquidation
    ///
    /// # Arguments
    ///
    /// * `params` - Optional query parameters for filtering by time, and pagination.
    ///
    /// # Returns
    ///
    /// A `Result` containing the positions under liquidation or an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # use orderly_connector_rs::types::GetPositionsUnderLiquidationParams;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// // Get all positions currently under liquidation (first page)
    /// let positions = service.get_positions_under_liquidation(None).await.expect("Failed");
    /// println!("Positions under liquidation: {:?}", positions);
    ///
    /// // Get positions with pagination
    /// let params = GetPositionsUnderLiquidationParams { page: Some(2), size: Some(10), ..Default::default() };
    /// let positions_page2 = service.get_positions_under_liquidation(Some(params)).await.expect("Failed");
    /// println!("Positions page 2: {:?}", positions_page2);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/public/get-positions-under-liquidation#get-positions-under-liquidation
    pub async fn get_positions_under_liquidation(
        &self,
        params: Option<GetPositionsUnderLiquidationParams>,
    ) -> Result<GetPositionsUnderLiquidationResponse> {
        let mut path = "/v1/public/liquidation".to_string();
        if let Some(p) = params {
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetPositionsUnderLiquidationParams to query string");
            }
        }
        let url = self.base_url.join(&path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Retrieves liquidated positions.
    ///
    /// Corresponds to GET /v1/public/liquidated_positions
    ///
    /// # Arguments
    ///
    /// * `params` - Optional query parameters for filtering by symbol, time, and pagination.
    ///
    /// # Returns
    ///
    /// A `Result` containing the liquidated positions or an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # use orderly_connector_rs::types::GetLiquidatedPositionsParams;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// // Get all liquidated positions (first page)
    /// let positions = service.get_liquidated_positions(None).await.expect("Failed");
    /// println!("Liquidated positions: {:?}", positions);
    ///
    /// // Get positions with pagination
    /// let params = GetLiquidatedPositionsParams { page: Some(2), size: Some(10), ..Default::default() };
    /// let positions_page2 = service.get_liquidated_positions(Some(params)).await.expect("Failed");
    /// println!("Liquidated positions page 2: {:?}", positions_page2);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/public/get-liquidated-positions-info
    pub async fn get_liquidated_positions(
        &self,
        params: Option<GetLiquidatedPositionsParams>,
    ) -> Result<GetLiquidatedPositionsResponse> {
        let mut path = "/v1/public/liquidated_positions".to_string();
        if let Some(p) = params {
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetLiquidatedPositionsParams to query string");
            }
        }
        let url = self.base_url.join(&path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Retrieves market trades for a given symbol.
    /// GET /v1/public/market_trades
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading pair symbol (e.g., "PERP_ETH_USDC")
    ///
    /// # Returns
    ///
    /// A `Result` containing the market trades or an error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orderly_connector_rs::rest::OrderlyService;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let service = OrderlyService::new(true, None).unwrap();
    /// let trades = service.get_market_trades("PERP_ETH_USDC").await.expect("Failed to get market trades");
    /// println!("Market trades: {:?}", trades);
    /// # }
    /// ```
    ///
    /// https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/public/get-market-trades
    pub async fn get_market_trades(&self, symbol: &str) -> Result<GetPublicTradesResponse> {
        let path = format!("/v1/public/market_trades?symbol={}&limit=500", symbol);
        let url = self.base_url.join(&path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    // --- Private Endpoints (Orders) ---

    /// Creates a new order for the specified user.
    /// Corresponds to POST /v1/order
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/create-order
    pub async fn create_order(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        order_req: CreateOrderRequest<'_>,
    ) -> Result<CreateOrderResponse> {
        let request = self
            .build_signed_request(creds, Method::POST, "/v1/order", Some(order_req)) // Pass creds
            .await?;
        self.send_request::<CreateOrderResponse>(request).await
    }

    /// Retrieves a specific order by its ID for the specified user.
    /// Corresponds to GET /v1/order/{order_id}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-order
    pub async fn get_order(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        order_id: u64,
    ) -> Result<GetOrderResponse> {
        let path = format!("/v1/order/{}", order_id);
        let request = self
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetOrderResponse>(request).await
    }

    /// Cancels an existing order by its ID for the specified user.
    /// Corresponds to DELETE /v1/order?order_id={order_id}&symbol={symbol}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/cancel-order
    pub async fn cancel_order(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        order_id: u64,
        symbol: &str,
    ) -> Result<CancelOrderResponse> {
        let path = format!("/v1/order?order_id={}&symbol={}", order_id, symbol);
        let request = self
            .build_signed_request::<()>(creds, Method::DELETE, &path, None) // Pass creds
            .await?;
        self.send_request::<CancelOrderResponse>(request).await
    }

    /// Retrieves multiple orders for the specified user based on filter parameters.
    /// Corresponds to GET /v1/orders
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-orders
    pub async fn get_orders(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        params: Option<GetOrdersParams<'_>>,
    ) -> Result<GetOrdersResponse> {
        let mut path = "/v1/orders".to_string();
        if let Some(p) = params {
            if let Ok(query) = serde_qs::to_string(&p) {
                if !query.is_empty() {
                    path.push('?');
                    path.push_str(&query);
                }
            } else {
                warn!("Failed to serialize GetOrdersParams to query string");
            }
        }
        let request = self
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetOrdersResponse>(request).await
    }

    // ===== Account Information =====

    /// Get current account information for the specified user.
    /// GET /v1/client/info
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-account-info
    pub async fn get_account_info(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
    ) -> Result<GetAccountInfoResponse> {
        let request = self
            .build_signed_request::<()>(creds, Method::GET, "/v1/client/info", None) // Pass creds
            .await?;
        self.send_request::<GetAccountInfoResponse>(request).await
    }

    // ===== Holdings / Balances =====

    /// Get current holdings (balances) for all tokens for the specified user.
    /// GET /v1/client/holding
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-holding
    pub async fn get_holding(&self, creds: &Credentials<'_>) -> Result<GetHoldingResponse> {
        // Added credentials parameter
        let request = self
            .build_signed_request::<()>(creds, Method::GET, "/v1/client/holding", None) // Pass creds
            .await?;
        self.send_request::<GetHoldingResponse>(request).await
    }

    // ===== Positions =====

    /// Get all current positions for the specified user.
    /// GET /v1/positions
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-all-positions-info
    pub async fn get_positions(&self, creds: &Credentials<'_>) -> Result<GetPositionsResponse> {
        // Added credentials parameter
        let request = self
            .build_signed_request::<()>(creds, Method::GET, "/v1/positions", None) // Pass creds
            .await?;
        self.send_request::<GetPositionsResponse>(request).await
    }

    /// Get position for a specific symbol for the specified user.
    /// GET /v1/position/{symbol}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-one-position-info
    pub async fn get_position(
        &self,
        creds: &Credentials<'_>,
        symbol: &str,
    ) -> Result<GetSinglePositionResponse> {
        // Added credentials parameter
        let path = format!("/v1/position/{}", symbol);
        let request = self
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetSinglePositionResponse>(request)
            .await
    }

    // ===== Asset History (Deposits/Withdrawals) =====

    /// Get asset history (deposits, withdrawals) for the specified user.
    /// GET /v1/asset/history
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-asset-history
    pub async fn get_asset_history(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        params: Option<GetAssetHistoryParams<'_>>,
    ) -> Result<GetAssetHistoryResponse> {
        let mut path = "/v1/asset/history".to_string();
        if let Some(p) = params {
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
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetAssetHistoryResponse>(request).await
    }

    // ===== Trades =====

    /// Get trade history for the specified user.
    /// GET /v1/trades
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-trades
    pub async fn get_trades(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        params: Option<GetTradesParams<'_>>,
    ) -> Result<GetTradesResponse> {
        let mut path = "/v1/trades".to_string();
        if let Some(p) = params {
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
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetTradesResponse>(request).await
    }

    /// Get specific trade by ID for the specified user.
    /// GET /v1/trade/{trade_id}
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-trade
    pub async fn get_trade(&self, creds: &Credentials<'_>, trade_id: u64) -> Result<Value> {
        // Added credentials parameter
        let path = format!("/v1/trade/{}", trade_id);
        let request = self
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<Value>(request).await
    }

    // ===== Client Statistics =====

    /// Get client statistics (e.g., 30d volume, VIP tier) for the specified user.
    /// GET /v1/client/statistics
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-client-statistics
    pub async fn get_client_statistics(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
    ) -> Result<GetClientStatisticsResponse> {
        let request = self
            .build_signed_request::<()>(creds, Method::GET, "/v1/client/statistics", None) // Pass creds
            .await?;
        self.send_request::<GetClientStatisticsResponse>(request)
            .await
    }

    // TODO: Implement endpoints for Liquidations, Fees, Settlement, Referrals, Broker, Delegate Signer, IP Restrictions etc.

    // ===== Withdrawals =====

    /// Request a withdrawal for the specified user.
    /// POST /v1/withdraw_request
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/request-withdrawal
    pub async fn request_withdrawal(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
        withdraw_req: WithdrawRequest<'_>,
    ) -> Result<WithdrawResponse> {
        let request = self
            .build_signed_request(
                creds,
                Method::POST,
                "/v1/withdraw_request",
                Some(withdraw_req),
            ) // Pass creds
            .await?;
        self.send_request::<WithdrawResponse>(request).await
    }
    // Note: Withdrawal history fetched via get_asset_history

    // ===== Fee Rates =====

    /// Get current fee rates for the specified user.
    /// GET /v1/client/fee_rates
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-fee-rates
    pub async fn get_fee_rates(&self, creds: &Credentials<'_>) -> Result<GetFeeRatesResponse> {
        // Added credentials parameter
        let request = self
            .build_signed_request::<()>(creds, Method::GET, "/v1/client/fee_rates", None) // Pass creds
            .await?;
        self.send_request::<GetFeeRatesResponse>(request).await
    }

    // ===== Liquidations =====

    /// Get liquidation history for the specified user's positions.
    /// GET /v1/liquidations
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-liquidations
    pub async fn get_liquidations(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
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
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetLiquidationsResponse>(request).await
    }

    // ===== PnL Settlement =====

    /// Get PnL settlement history for the specified user.
    /// GET /v1/settlements
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-settlement-history
    pub async fn get_settlement_history(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
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
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetSettlementsResponse>(request).await
    }

    // ===== Funding Fee =====

    /// Get funding fee history for the specified user.
    /// GET /v1/funding_fee/history
    ///
    /// https://orderly.network/docs/build-on-evm/evm-api/restful-api/private/get-funding-fee-history
    pub async fn get_funding_fee_history(
        &self,
        creds: &Credentials<'_>, // Added credentials parameter
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
            .build_signed_request::<()>(creds, Method::GET, &path, None) // Pass creds
            .await?;
        self.send_request::<GetFundingFeeHistoryResponse>(request)
            .await
    }

    // ===== Algo Orders =====

    /// Creates a new algorithmic order.
    ///
    /// # Arguments
    ///
    /// * `creds` - Credentials for authentication
    /// * `request` - Details of the algo order to create
    ///
    /// # Returns
    ///
    /// A `SuccessResponse` containing the details of the created algo order or an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orderly_connector_rs::{
    ///     rest::{Client, Credentials},
    ///     types::{AlgoOrderType, CreateAlgoOrderRequest, Side},
    ///     error::OrderlyError
    /// };
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Replace with actual credentials or load from env
    ///     let creds = Credentials {
    ///         orderly_key: "your_api_key",
    ///         orderly_secret: "your_api_secret",
    ///         orderly_account_id: "your_account_id",
    ///     };
    ///     let service = Client::new("https://testnet-api-evm.orderly.network", None)?;
    ///
    ///     let request = CreateAlgoOrderRequest {
    ///         symbol: "PERP_BTC_USDC".to_string(),
    ///         order_type: AlgoOrderType::StopMarket,
    ///         side: Side::Sell,
    ///         quantity: 0.1,
    ///         trigger_price: 50000.0,
    ///         limit_price: None,
    ///         trailing_delta: None,
    ///         client_order_id: Some("my_stop_loss_1".to_string()),
    ///         reduce_only: Some(true),
    ///     };
    ///
    ///     match service.create_algo_order(&creds, request).await {
    ///         Ok(response) => println!("Created algo order: {:?}", response.data),
    ///         Err(e) => println!("Failed to create algo order: {}", e),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_algo_order(
        &self,
        creds: &Credentials<'_>,
        request: CreateAlgoOrderRequest,
    ) -> Result<SuccessResponse<AlgoOrderDetails>, OrderlyError> {
        // Validate request
        if request.symbol.is_empty() {
            return Err(OrderlyError::ValidationError(
                "Symbol is required".to_string(),
            ));
        }
        if request.quantity <= 0.0 {
            return Err(OrderlyError::ValidationError(
                "Quantity must be positive".to_string(),
            ));
        }
        if request.trigger_price <= 0.0 {
            return Err(OrderlyError::ValidationError(
                "Trigger price must be positive".to_string(),
            ));
        }
        if let Some(limit_price) = request.limit_price {
            if limit_price <= 0.0 {
                return Err(OrderlyError::ValidationError(
                    "Limit price must be positive if provided".to_string(),
                ));
            }
        }

        // Build path
        let path = "/v1/algo-order";

        // Build signed request
        let signed_request = self
            .build_signed_request(creds, Method::POST, path, Some(&request))
            .await?;

        // Send request and handle response
        self.send_request::<SuccessResponse<AlgoOrderDetails>>(signed_request)
            .await
    }

    /// Cancels an existing algorithmic order
    ///
    /// # Arguments
    ///
    /// * `creds` - Credentials for authentication
    /// * `symbol` - Trading pair symbol
    /// * `algo_order_id` - ID of the algo order to cancel
    ///
    /// # Returns
    ///
    /// A `SuccessResponse` containing the details of the cancelled algo order or an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orderly_connector_rs::rest::{Client, Credentials};
    /// use orderly_connector_rs::error::OrderlyError;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let creds = Credentials {
    ///         orderly_key: "your_api_key",
    ///         orderly_secret: "your_api_secret",
    ///         orderly_account_id: "your_account_id",
    ///     };
    ///     let service = Client::new("https://testnet-api-evm.orderly.network", None)?;
    ///
    ///     match service.cancel_algo_order(&creds, "PERP_BTC_USDC", "123456").await {
    ///         Ok(response) => println!("Cancelled algo order: {:?}", response.data),
    ///         Err(e) => println!("Failed to cancel algo order: {}", e),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn cancel_algo_order(
        &self,
        creds: &Credentials<'_>,
        symbol: &str,
        algo_order_id: &str,
    ) -> Result<SuccessResponse<AlgoOrderDetails>, OrderlyError> {
        // Validate inputs
        if symbol.is_empty() {
            return Err(OrderlyError::ValidationError(
                "Symbol is required".to_string(),
            ));
        }
        if algo_order_id.is_empty() {
            return Err(OrderlyError::ValidationError(
                "Algo order ID is required".to_string(),
            ));
        }

        // Build path
        let path = format!("/v1/algo-order/{}/{}", symbol, algo_order_id);

        // Build signed request
        let request = self
            .build_signed_request::<()>(creds, Method::DELETE, &path, None)
            .await?;

        // Send request and handle response
        self.send_request::<SuccessResponse<AlgoOrderDetails>>(request)
            .await
    }

    /// Cancels an algorithmic order by its client order ID
    ///
    /// # Arguments
    ///
    /// * `creds` - Credentials for authentication
    /// * `symbol` - Trading pair symbol
    /// * `client_order_id` - Client-provided order ID
    ///
    /// # Returns
    ///
    /// A `SuccessResponse` containing the details of the cancelled algo order or an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orderly_connector_rs::rest::{Client, Credentials};
    /// use orderly_connector_rs::error::OrderlyError;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let creds = Credentials {
    ///         orderly_key: "your_api_key",
    ///         orderly_secret: "your_api_secret",
    ///         orderly_account_id: "your_account_id",
    ///     };
    ///     let service = Client::new("https://testnet-api-evm.orderly.network", None)?;
    ///
    ///     match service.cancel_algo_order_by_client_id(&creds, "PERP_BTC_USDC", "my_stop_loss_1").await {
    ///         Ok(response) => println!("Cancelled algo order: {:?}", response.data),
    ///         Err(e) => println!("Failed to cancel algo order by client ID: {}", e),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn cancel_algo_order_by_client_id(
        &self,
        creds: &Credentials<'_>,
        symbol: &str,
        client_order_id: &str,
    ) -> Result<SuccessResponse<AlgoOrderDetails>, OrderlyError> {
        // Validate inputs
        if symbol.is_empty() {
            return Err(OrderlyError::ValidationError(
                "Symbol is required".to_string(),
            ));
        }
        if client_order_id.is_empty() {
            return Err(OrderlyError::ValidationError(
                "Client order ID is required".to_string(),
            ));
        }

        // Build path
        let path = format!(
            "/v1/algo-order/{}/by-client-order-id/{}",
            symbol, client_order_id
        );

        // Build signed request
        let request = self
            .build_signed_request::<()>(creds, Method::DELETE, &path, None)
            .await?;

        // Send request and handle response
        self.send_request::<SuccessResponse<AlgoOrderDetails>>(request)
            .await
    }

    /// Gets a list of algorithmic orders with optional filtering
    ///
    /// # Arguments
    ///
    /// * `creds` - Credentials for authentication
    /// * `params` - Optional query parameters to filter the results
    ///
    /// # Returns
    ///
    /// A `SuccessResponse` containing a paginated list of algo orders or an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orderly_connector_rs::{
    ///     rest::{Client, Credentials},
    ///     types::{AlgoOrderType, GetAlgoOrdersParams},
    ///     error::OrderlyError
    /// };
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let creds = Credentials {
    ///         orderly_key: "your_api_key",
    ///         orderly_secret: "your_api_secret",
    ///         orderly_account_id: "your_account_id",
    ///     };
    ///     let service = Client::new("https://testnet-api-evm.orderly.network", None)?;
    ///
    ///     // Get all algo orders
    ///     match service.get_algo_orders(&creds, GetAlgoOrdersParams::default()).await {
    ///         Ok(response) => println!("Found {} algo orders", response.data.rows.len()),
    ///         Err(e) => println!("Failed to get algo orders: {}", e),
    ///     }
    ///
    ///     // Get algo orders with filters
    ///     let params = GetAlgoOrdersParams {
    ///         symbol: Some("PERP_BTC_USDC".to_string()),
    ///         status: Some("NEW".to_string()),
    ///         side: Some("SELL".to_string()),
    ///         order_type: Some(AlgoOrderType::StopMarket),
    ///         page: Some(1),
    ///         size: Some(10),
    ///     };
    ///
    ///     match service.get_algo_orders(&creds, params).await {
    ///         Ok(response) => println!("Found {} stop market sell orders", response.data.rows.len()),
    ///         Err(e) => println!("Failed to get algo orders with filters: {}", e),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_algo_orders(
        &self,
        creds: &Credentials<'_>,
        params: GetAlgoOrdersParams,
    ) -> Result<SuccessResponse<GetAlgoOrdersResponse>, OrderlyError> {
        // Convert params to query string
        let query_string = serde_qs::to_string(&params)
            .map_err(|e| OrderlyError::JsonEncodeError(e.to_string()))?;

        // Build path
        let path = if query_string.is_empty() {
            "/v1/algo-orders".to_string()
        } else {
            format!("/v1/algo-orders?{}", query_string)
        };

        // Build signed request
        let request = self
            .build_signed_request::<()>(creds, Method::GET, &path, None)
            .await?;

        // Send request and handle response
        self.send_request::<SuccessResponse<GetAlgoOrdersResponse>>(request)
            .await
    }
}

// ===== Helper Structs (Restore these) =====

#[derive(Deserialize, Debug, Clone)]
pub struct SymbolInfo {
    pub symbol: String,
    pub quote_min: f64,
    pub quote_max: f64,
    pub quote_tick: f64,
    pub base_min: f64,
    pub base_max: f64,
    pub base_tick: f64,
    pub min_notional: f64,
    pub price_range: f64,
    pub created_time: u64,
    pub updated_time: u64,
    pub imr_factor: Option<f64>,
    pub liquidation_fee: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AllSymbolsData {
    pub rows: Vec<SymbolInfo>,
}

// Use an enum to represent the two possible structures of the 'data' field
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)] // Important for trying to deserialize as either variant
pub enum ExchangeInfoData {
    Single(SymbolInfo),
    All(AllSymbolsData),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExchangeInfoResponse {
    pub success: bool,
    pub timestamp: u64,
    pub data: ExchangeInfoData,
}
