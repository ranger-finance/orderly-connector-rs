use crate::auth::{self, get_timestamp_ms};
use crate::error::{OrderlyError, Result};
use crate::types::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use reqwest::{Client as HttpClient, Method, Request, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use url::Url;

const MAINNET_API_URL: &str = "https://api-evm.orderly.network";
const TESTNET_API_URL: &str = "https://testnet-api-evm.orderly.network";
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

#[derive(Clone)]
pub struct Client {
    http_client: HttpClient,
    base_url: Url,
    orderly_key: String,
    orderly_secret: String,
    orderly_account_id: String,
    // timeout is configured directly in the HttpClient
}

impl Client {
    /// Creates a new Orderly REST API client.
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
    /// Corresponds to GET /v1/public/system_info
    pub async fn get_system_status(&self) -> Result<Value> {
        let path = "/v1/public/system_info";
        let url = self.base_url.join(path)?;
        let request = self.http_client.get(url).build()?;
        self.send_public_request(request).await
    }

    /// Retrieves exchange information, optionally filtered by symbol.
    /// Corresponds to GET /v1/public/info and GET /v1/public/info/{symbol}
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
    pub async fn create_order(&self, order: CreateOrderRequest<'_>) -> Result<CreateOrderResponse> {
        let path = "/v1/order";
        let request = self
            .build_signed_request(Method::POST, path, Some(order))
            .await?;
        let response = self.http_client.execute(request).await?;
        Self::handle_response(response).await
    }

    /// Retrieves a specific order by its ID.
    /// Corresponds to GET /v1/order/{order_id}
    pub async fn get_order(&self, order_id: u64) -> Result<GetOrderResponse> {
        let path = format!("/v1/order/{}", order_id);
        let request = self
            .build_signed_request::<()>(Method::GET, &path, None)
            .await?;
        let response = self.http_client.execute(request).await?;
        Self::handle_response(response).await
    }

    /// Cancels an existing order by its ID.
    /// Corresponds to DELETE /v1/order?order_id={order_id}&symbol={symbol}
    pub async fn cancel_order(&self, order_id: u64, symbol: &str) -> Result<CancelOrderResponse> {
        let path_and_query = format!("/v1/order?order_id={}&symbol={}", order_id, symbol);
        let request = self
            .build_signed_request::<()>(Method::DELETE, &path_and_query, None)
            .await?;
        let response = self.http_client.execute(request).await?;
        Self::handle_response(response).await
    }

    /// Retrieves multiple orders based on filter parameters.
    /// Corresponds to GET /v1/orders
    pub async fn get_orders(
        &self,
        params: Option<GetOrdersParams<'_>>,
    ) -> Result<GetOrdersResponse> {
        let path = "/v1/orders";
        let mut full_url = self.base_url.join(path)?;

        // Serialize params to query string and add to the URL
        if let Some(p) = params {
            let query = serde_qs::to_string(&p).map_err(|e| OrderlyError::ParameterTypeError {
                param: "query_params".to_string(),
                expected: "valid query params".to_string(),
                received: e.to_string(),
            })?;
            if !query.is_empty() {
                full_url.set_query(Some(&query));
            }
        }

        // Extract path and query for signing (e.g., "/v1/orders?symbol=PERP_ETH_USDC")
        let path_and_query = full_url.path().to_string()
            + &full_url
                .query()
                .map_or_else(String::new, |q| format!("?{}", q));

        // Build the request using the path+query for signing, but the full URL for execution
        let request_to_sign = self
            .build_signed_request::<()>(Method::GET, &path_and_query, None)
            .await?;

        // Execute the request using the headers from the signed request but the *full* URL
        let final_request = self
            .http_client
            .request(Method::GET, full_url)
            .headers(request_to_sign.headers().clone())
            .build()?;

        let response = self.http_client.execute(final_request).await?;
        Self::handle_response(response).await
    }
}
