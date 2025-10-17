//! Test fixtures for authentication, orders, and mock data
//!
//! Provides reusable test data structures including:
//! - TestCredentials: Binance Testnet API keys
//! - TestBearerToken: HTTP Authorization bearer tokens
//! - SampleOrder: Predefined order structures for testing

use std::env;

/// Test credentials for Binance Testnet API
#[derive(Debug, Clone)]
pub struct TestCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
}

impl TestCredentials {
    /// Load credentials from environment variables
    /// Falls back to mock values if env vars not set
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("BINANCE_TESTNET_API_KEY")
                .unwrap_or_else(|_| "test_api_key".to_string()),
            api_secret: env::var("BINANCE_TESTNET_API_SECRET")
                .unwrap_or_else(|_| "test_api_secret".to_string()),
            base_url: env::var("BINANCE_TESTNET_BASE_URL")
                .unwrap_or_else(|_| "https://testnet.binance.vision".to_string()),
        }
    }

    /// Create mock credentials for testing without API calls
    pub fn mock() -> Self {
        Self {
            api_key: "mock_api_key_12345".to_string(),
            api_secret: "mock_api_secret_67890".to_string(),
            base_url: "http://localhost:8080".to_string(),
        }
    }
}

/// Bearer token for HTTP Authorization header
/// Note: bearer token = HTTP_BEARER_TOKEN env var
#[derive(Debug, Clone)]
pub struct TestBearerToken {
    pub token: String,
}

impl TestBearerToken {
    /// Create valid bearer token from environment
    pub fn valid() -> Self {
        Self {
            token: env::var("HTTP_BEARER_TOKEN").unwrap_or_else(|_| "test_token_123".to_string()),
        }
    }

    /// Create expired/invalid bearer token for testing rejection
    pub fn expired() -> Self {
        Self {
            token: "expired_token_invalid".to_string(),
        }
    }

    /// Format as Authorization header value
    pub fn as_header(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

/// Sample order structures for testing order management endpoints
#[derive(Debug, Clone)]
pub struct SampleOrder {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    pub time_in_force: Option<String>,
}

impl SampleOrder {
    /// Create market buy order (immediate execution at market price)
    pub fn market_buy() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: "0.001".to_string(),
            price: None,
            time_in_force: None,
        }
    }

    /// Create limit sell order (waits for specified price)
    pub fn limit_sell() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: "0.001".to_string(),
            price: Some("50000.00".to_string()),
            time_in_force: Some("GTC".to_string()), // Good Till Cancel
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_from_env() {
        let creds = TestCredentials::from_env();
        assert!(!creds.api_key.is_empty());
        assert!(!creds.api_secret.is_empty());
        assert!(!creds.base_url.is_empty());
    }

    #[test]
    fn test_credentials_mock() {
        let creds = TestCredentials::mock();
        assert_eq!(creds.api_key, "mock_api_key_12345");
        assert_eq!(creds.api_secret, "mock_api_secret_67890");
    }

    #[test]
    fn test_bearer_token_valid() {
        let token = TestBearerToken::valid();
        assert!(!token.token.is_empty());
        assert!(token.as_header().starts_with("Bearer "));
    }

    #[test]
    fn test_bearer_token_expired() {
        let token = TestBearerToken::expired();
        assert_eq!(token.token, "expired_token_invalid");
    }

    #[test]
    fn test_sample_order_market_buy() {
        let order = SampleOrder::market_buy();
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, "BUY");
        assert_eq!(order.order_type, "MARKET");
        assert!(order.price.is_none());
    }

    #[test]
    fn test_sample_order_limit_sell() {
        let order = SampleOrder::limit_sell();
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, "SELL");
        assert_eq!(order.order_type, "LIMIT");
        assert!(order.price.is_some());
        assert_eq!(order.time_in_force.as_deref(), Some("GTC"));
    }
}
