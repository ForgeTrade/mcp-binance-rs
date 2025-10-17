//! Error Types and Handling
//!
//! Defines error types used throughout the MCP server with secure error messages
//! that never expose sensitive data.

use std::time::Duration;
use thiserror::Error;

/// Enhanced error types for MCP protocol with actionable recovery suggestions
///
/// This enum provides structured error information with user-friendly messages and
/// recovery suggestions for common error scenarios.
#[derive(Debug, Error)]
pub enum BinanceError {
    /// Rate limit exceeded error with retry information
    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimited {
        retry_after: Duration,
        current_weight: u32,
        weight_limit: u32,
    },

    /// Invalid API credentials error with masked key for debugging
    #[error("Invalid API credentials. Check environment variables")]
    InvalidCredentials {
        masked_key: String,
        help_url: String,
    },

    /// Invalid trading symbol error with format guidance
    #[error("Invalid trading symbol: {provided}")]
    InvalidSymbol {
        provided: String,
        format_help: String,
        examples: Vec<String>,
    },

    /// Insufficient balance error with detailed amounts
    #[error("Insufficient {asset} balance")]
    InsufficientBalance {
        asset: String,
        required: String,
        available: String,
    },

    /// Wrapper for existing API errors (backward compatibility)
    #[error("Binance API error: {0}")]
    ApiError(#[from] reqwest::Error),
}

/// Masks an API key for safe error reporting
///
/// Returns a string showing only the first 4 and last 4 characters, with asterisks
/// in between. For keys shorter than 8 characters, returns all asterisks.
///
/// # Examples
///
/// ```
/// use mcp_binance_server::error::mask_api_key;
/// assert_eq!(mask_api_key("AbCdEfGhIjKlMnOpQrStUvWxYz"), "AbCd****WxYz");
/// assert_eq!(mask_api_key("short"), "*****");
/// ```
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

/// Main error type for MCP Binance Server
///
/// All errors in the system are represented by this enum. Error messages are
/// designed to be user-friendly and never contain sensitive data like API keys
/// or internal state.
#[derive(Error, Debug)]
pub enum McpError {
    /// Network failures or connectivity issues with Binance API
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// HTTP 429 responses from Binance (rate limit exceeded)
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// JSON deserialization or parsing failures
    #[error("Parse error: {0}")]
    ParseError(String),

    /// MCP protocol violations or invalid requests
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Server not fully initialized
    #[error("Server not ready: {0}")]
    NotReady(String),

    /// Unexpected internal errors
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl McpError {
    /// Returns true if this error type should trigger retry logic
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            McpError::ConnectionError(_) | McpError::RateLimitError(_)
        )
    }

    /// Returns error type string for MCP protocol responses
    pub fn error_type(&self) -> &'static str {
        match self {
            McpError::ConnectionError(_) => "connection_error",
            McpError::RateLimitError(_) => "rate_limit",
            McpError::ParseError(_) => "parse_error",
            McpError::InvalidRequest(_) => "invalid_request",
            McpError::NotReady(_) => "not_ready",
            McpError::InternalError(_) => "internal_error",
        }
    }
}

// Error conversions from common error types
impl From<reqwest::Error> for McpError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            McpError::ConnectionError(
                "Request timeout. Please check your internet connection.".to_string(),
            )
        } else if err.is_connect() {
            McpError::ConnectionError(
                "Failed to connect to Binance API. Please check your internet connection."
                    .to_string(),
            )
        } else if let Some(status) = err.status() {
            match status.as_u16() {
                429 => McpError::RateLimitError(
                    "Too many requests to Binance API. Retry after 60 seconds.".to_string(),
                ),
                418 => McpError::ConnectionError(
                    "IP address banned by Binance. Please contact support.".to_string(),
                ),
                403 => McpError::ConnectionError(
                    "WAF limit violated. Please reduce request frequency.".to_string(),
                ),
                500..=599 => McpError::ConnectionError(format!(
                    "Binance server error (HTTP {}). Please try again later.",
                    status.as_u16()
                )),
                400..=499 => McpError::InvalidRequest(format!(
                    "Invalid request (HTTP {}). Please check parameters.",
                    status.as_u16()
                )),
                _ => McpError::InternalError(format!("HTTP error: {}", status.as_u16())),
            }
        } else {
            McpError::ConnectionError(format!(
                "Network error: {}. Please check your connection.",
                err
            ))
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::ParseError(format!("Failed to parse JSON response: {}", err))
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::InternalError(format!("I/O error: {}", err))
    }
}

// HTTP response conversion for axum
#[cfg(feature = "http-api")]
impl axum::response::IntoResponse for McpError {
    fn into_response(self) -> axum::response::Response {
        use axum::Json;
        use axum::http::StatusCode;
        use serde_json::json;

        let (status, error_type, message) = match &self {
            McpError::ConnectionError(_) => {
                (StatusCode::BAD_GATEWAY, self.error_type(), self.to_string())
            }
            McpError::RateLimitError(_) => (
                StatusCode::TOO_MANY_REQUESTS,
                self.error_type(),
                self.to_string(),
            ),
            McpError::ParseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                self.error_type(),
                "Failed to parse API response".to_string(),
            ),
            McpError::InvalidRequest(_) => {
                (StatusCode::BAD_REQUEST, self.error_type(), self.to_string())
            }
            McpError::NotReady(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                self.error_type(),
                self.to_string(),
            ),
            McpError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                self.error_type(),
                "An internal error occurred".to_string(),
            ),
        };

        let body = Json(json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// MCP ErrorData conversion for enhanced error reporting
impl From<BinanceError> for rmcp::ErrorData {
    fn from(err: BinanceError) -> Self {
        use rmcp::model::ErrorCode;
        use serde_json::json;

        match err {
            BinanceError::RateLimited { retry_after, current_weight, weight_limit } => {
                rmcp::ErrorData::new(
                    ErrorCode(-32001),
                    format!("Rate limit exceeded. Please wait {} seconds before retrying.", retry_after.as_secs()),
                    Some(json!({
                        "retry_after_secs": retry_after.as_secs(),
                        "current_weight": current_weight,
                        "weight_limit": weight_limit,
                        "recovery_suggestion": "Reduce request frequency or wait for rate limit window to reset"
                    }))
                )
            },

            BinanceError::InvalidCredentials { masked_key, help_url } => {
                rmcp::ErrorData::new(
                    ErrorCode(-32002),
                    "Invalid API credentials. Please check your BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables.".to_string(),
                    Some(json!({
                        "masked_api_key": masked_key,
                        "help_url": help_url,
                        "recovery_suggestion": "Verify credentials at https://testnet.binance.vision/ and ensure correct environment variables"
                    }))
                )
            },

            BinanceError::InvalidSymbol { provided, format_help, examples } => {
                rmcp::ErrorData::new(
                    ErrorCode(-32003),
                    format!("Invalid trading symbol '{}'. {}", provided, format_help),
                    Some(json!({
                        "provided_symbol": provided,
                        "valid_examples": examples,
                        "recovery_suggestion": "Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT)"
                    }))
                )
            },

            BinanceError::InsufficientBalance { asset, required, available } => {
                rmcp::ErrorData::new(
                    ErrorCode(-32004),
                    format!("Insufficient {} balance. Required: {}, Available: {}", asset, required, available),
                    Some(json!({
                        "asset": asset,
                        "required_amount": required,
                        "available_amount": available,
                        "recovery_suggestion": "Deposit more funds or reduce order quantity"
                    }))
                )
            },

            BinanceError::ApiError(e) => {
                rmcp::ErrorData::internal_error(format!("Binance API error: {}", e), None)
            },
        }
    }
}
