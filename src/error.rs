//! Error Types and Handling
//!
//! Defines error types used throughout the MCP server with secure error messages
//! that never expose sensitive data.

use thiserror::Error;

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
