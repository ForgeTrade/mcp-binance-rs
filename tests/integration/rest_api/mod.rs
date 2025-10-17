//! REST API integration tests
//!
//! Comprehensive test suite for all 15 REST API endpoints:
//! - Market Data: ticker, depth, trades, klines, avgPrice
//! - Orders: place, query, cancel, openOrders, allOrders
//! - Account: account, myTrades, rateLimit, userDataStream (start/close)

pub mod account;
pub mod market_data;
pub mod orders;

use crate::common::{binance_client, fixtures::TestCredentials};
use std::time::Duration;

/// Helper: Create authenticated test client with credentials
pub fn setup_authenticated_client() -> (reqwest::Client, TestCredentials) {
    let creds = TestCredentials::from_env();
    let client = binance_client::create_testnet_client(&creds);
    (client, creds)
}

/// Helper: Create public test client (no authentication)
pub fn setup_public_client() -> reqwest::Client {
    binance_client::create_public_client()
}

/// Helper: Sign request parameters for authenticated endpoints
/// Uses HMAC-SHA256 signature as required by Binance API
pub fn sign_request_params(query_string: &str, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(query_string.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Helper: Build authenticated request URL with timestamp and signature
pub fn build_authenticated_url(
    base_url: &str,
    endpoint: &str,
    params: &[(&str, &str)],
    secret: &str,
) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let mut query_params: Vec<String> =
        params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();

    query_params.push(format!("timestamp={}", timestamp));

    let query_string = query_params.join("&");
    let signature = sign_request_params(&query_string, secret);

    format!(
        "{}/api/v3/{}?{}&signature={}",
        base_url, endpoint, query_string, signature
    )
}

/// Helper: Assert CORS headers are present in response
pub fn assert_cors_headers(response: &reqwest::Response) {
    let headers = response.headers();

    assert!(
        headers.contains_key("access-control-allow-origin"),
        "Missing CORS header: Access-Control-Allow-Origin"
    );

    // Verify wildcard or specific origin
    if let Some(origin) = headers.get("access-control-allow-origin") {
        let origin_str = origin.to_str().unwrap_or("");
        assert!(
            origin_str == "*" || !origin_str.is_empty(),
            "Invalid Access-Control-Allow-Origin value"
        );
    }
}

/// Helper: Wait for rate limit window to reset (Binance has weight limits)
pub async fn wait_for_rate_limit() {
    tokio::time::sleep(Duration::from_millis(500)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request_params() {
        let query = "symbol=BTCUSDT&timestamp=1234567890";
        let secret = "test_secret";
        let signature = sign_request_params(query, secret);

        // Should produce consistent hex-encoded HMAC-SHA256
        assert_eq!(signature.len(), 64); // 32 bytes = 64 hex chars
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_build_authenticated_url() {
        let url = build_authenticated_url(
            "https://testnet.binance.vision",
            "account",
            &[("recvWindow", "5000")],
            "test_secret",
        );

        assert!(url.contains("https://testnet.binance.vision/api/v3/account?"));
        assert!(url.contains("recvWindow=5000"));
        assert!(url.contains("timestamp="));
        assert!(url.contains("signature="));
    }
}
