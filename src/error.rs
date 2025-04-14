use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;
use url::ParseError;

/// A type alias for `Result<T, OrderlyError>`.
pub type Result<T, E = OrderlyError> = std::result::Result<T, E>;

/// The main error type for the Orderly Network client.
///
/// This enum represents all possible errors that can occur when interacting with
/// the Orderly Network API, including client errors, server errors, and various
/// other error conditions.
///
/// # Examples
///
/// ```no_run
/// use orderly_connector_rs::error::{OrderlyError, Result};
///
/// fn handle_error(result: Result<()>) {
///     match result {
///         Ok(_) => println!("Success!"),
///         Err(OrderlyError::ClientError { status, code, message, .. }) => {
///             println!("Client error: {} (status: {}, code: {})", message, status, code);
///         },
///         Err(e) => println!("Other error: {}", e),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum OrderlyError {
    /// Represents errors returned by the Orderly API when the request is invalid.
    ///
    /// This includes errors like invalid parameters, authentication failures,
    /// and other client-side issues.
    #[error("Client Error: status={status}, code={code}, message={message}, data={data:?}")]
    ClientError {
        /// The HTTP status code returned by the server
        status: StatusCode,
        /// The error code specific to the Orderly API
        code: i64,
        /// A human-readable error message
        message: String,
        /// Optional additional error data
        data: Option<Value>,
        /// The HTTP headers from the response
        header: HeaderMap,
    },

    /// Represents errors returned by the Orderly API when there's a server-side issue.
    ///
    /// This includes errors like internal server errors, service unavailability,
    /// and other server-side issues.
    #[error("Server Error: status={status}, code={code}, message={message}")]
    ServerError {
        /// The HTTP status code returned by the server
        status: StatusCode,
        /// The error code specific to the Orderly API
        code: i64,
        /// A human-readable error message
        message: String,
        /// The HTTP headers from the response
        header: HeaderMap,
    },

    /// Indicates that a required parameter was missing from a request.
    #[error("Parameter Required Error: Missing required parameter '{param}'")]
    ParameterRequiredError {
        /// The name of the missing parameter
        param: String,
    },

    /// Indicates that a parameter had an invalid value.
    #[error("Parameter Value Error: Invalid value '{value}' for parameter '{param}'. Allowed values: {allowed:?}")]
    ParameterValueError {
        /// The name of the parameter with an invalid value
        param: String,
        /// The invalid value that was provided
        value: String,
        /// The list of allowed values for this parameter
        allowed: Vec<String>,
    },

    /// Indicates that a parameter had an invalid type.
    #[error("Parameter Type Error: Invalid type for parameter '{param}'. Expected {expected}, received {received}")]
    ParameterTypeError {
        /// The name of the parameter with an invalid type
        param: String,
        /// The expected type for this parameter
        expected: String,
        /// The actual type that was provided
        received: String,
    },

    /// Represents errors that occur during WebSocket operations.
    #[error("WebSocket Error: {0}")]
    WebsocketError(String),

    /// Represents errors that occur during authentication.
    #[error("Authentication Error: {0}")]
    AuthenticationError(String),

    /// Represents errors from the HTTP client (reqwest).
    #[error("HTTP Request Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Represents errors during JSON serialization or deserialization.
    #[error("JSON Deserialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Represents errors during URL parsing.
    #[error("URL Parsing Error: {0}")]
    UrlParseError(#[from] ParseError),

    /// Represents errors with HTTP header values.
    #[error("Invalid HTTP Header Value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    /// Represents errors when getting the system time.
    #[error("System Time Error: {0}")]
    TimestampError(#[from] std::time::SystemTimeError),

    /// Represents errors during Ed25519 signature operations.
    #[error("Ed25519 Signature Error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::SignatureError),

    /// Represents errors during I/O operations.
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents network errors.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Represents JSON encoding errors.
    #[error("JSON encode error: {0}")]
    JsonEncodeError(String),

    /// Represents validation errors.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Represents missing credentials errors.
    #[error("Missing credentials")]
    MissingCredentials,
}

impl From<bs58::decode::Error> for OrderlyError {
    fn from(err: bs58::decode::Error) -> Self {
        OrderlyError::AuthenticationError(format!("Failed to decode base58 secret key: {}", err))
    }
}
