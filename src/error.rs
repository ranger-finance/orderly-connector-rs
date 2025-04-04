use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;
use url::ParseError;

pub type Result<T, E = OrderlyError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum OrderlyError {
    #[error("Client Error: status={status}, code={code}, message={message}, data={data:?}")]
    ClientError {
        status: StatusCode,
        code: i64,
        message: String,
        data: Option<Value>,
        header: HeaderMap,
    },

    #[error("Server Error: status={status}, code={code}, message={message}")]
    ServerError {
        status: StatusCode,
        code: i64,
        message: String,
        header: HeaderMap,
    },

    #[error("Parameter Required Error: Missing required parameter '{param}'")]
    ParameterRequiredError { param: String },

    #[error("Parameter Value Error: Invalid value '{value}' for parameter '{param}'. Allowed values: {allowed:?}")]
    ParameterValueError {
        param: String,
        value: String,
        allowed: Vec<String>,
    },

    #[error("Parameter Type Error: Invalid type for parameter '{param}'. Expected {expected}, received {received}")]
    ParameterTypeError {
        param: String,
        expected: String,
        received: String,
    },

    #[error("WebSocket Error: {0}")]
    WebsocketError(String),

    #[error("Authentication Error: {0}")]
    AuthenticationError(String),

    #[error("HTTP Request Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON Serialization/Deserialization Error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("URL Parsing Error: {0}")]
    UrlParseError(#[from] ParseError),

    #[error("Invalid HTTP Header Value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("System Time Error: {0}")]
    TimestampError(#[from] std::time::SystemTimeError),

    #[error("Ed25519 Signature Error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::SignatureError),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}
