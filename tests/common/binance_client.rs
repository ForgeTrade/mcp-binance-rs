//! Binance Testnet client configuration helpers
//!
//! Provides utilities for creating configured HTTP clients
//! that connect to Binance Testnet API with proper timeouts
//! and authentication headers

use std::time::Duration;

use super::fixtures::TestCredentials;

/// Create reqwest client configured for Binance Testnet
/// with appropriate timeouts and authentication
pub fn create_testnet_client(creds: &TestCredentials) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "X-MBX-APIKEY",
                reqwest::header::HeaderValue::from_str(&creds.api_key).expect("Invalid API key"),
            );
            headers
        })
        .build()
        .expect("Failed to build HTTP client")
}

/// Create reqwest client without authentication (for public endpoints)
pub fn create_public_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_testnet_client() {
        let creds = TestCredentials::mock();
        let _client = create_testnet_client(&creds);
        // Client created successfully (test passes if no panic)
    }

    #[test]
    fn test_create_public_client() {
        let _client = create_public_client();
        // Client created successfully (test passes if no panic)
    }
}
