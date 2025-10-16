//! Binance HTTP Client
//!
//! HTTP client wrapper for making requests to Binance REST API.
//! Provides timeout configuration and user-agent headers.

use crate::binance::types::ServerTimeResponse;
use crate::error::McpError;
use reqwest::Client;
use std::time::Duration;

/// Binance REST API HTTP client
///
/// Wraps reqwest::Client with Binance-specific configuration including
/// timeouts, base URL, and user-agent headers.
#[derive(Clone, Debug)]
pub struct BinanceClient {
    /// HTTP client for making requests
    pub(crate) client: Client,
    /// Base URL for Binance API (default: https://api.binance.com)
    pub(crate) base_url: String,
}

impl BinanceClient {
    /// Creates a new Binance client with default settings
    ///
    /// Default configuration:
    /// - Base URL: https://api.binance.com
    /// - Timeout: 10 seconds
    /// - User-Agent: mcp-binance-server/0.1.0
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(10))
    }

    /// Creates a new Binance client with custom timeout
    ///
    /// # Arguments
    /// * `timeout` - Request timeout duration
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use mcp_binance_server::binance::client::BinanceClient;
    ///
    /// let client = BinanceClient::with_timeout(Duration::from_secs(5));
    /// ```
    pub fn with_timeout(timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent("mcp-binance-server/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://api.binance.com".to_string(),
        }
    }

    /// Returns the configured base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Fetches current Binance server time
    ///
    /// Calls GET /api/v3/time endpoint and returns the server timestamp in milliseconds.
    /// Implements exponential backoff for rate limit (429) responses with up to 3 retries.
    ///
    /// # Returns
    /// * `Ok(i64)` - Server time in milliseconds since Unix epoch
    /// * `Err(McpError)` - Network error, rate limit exceeded, or parse error
    ///
    /// # Errors
    /// * `ConnectionError` - Network failures, timeouts, 5xx server errors
    /// * `RateLimitError` - HTTP 429 after max retries (3 attempts)
    /// * `ParseError` - Invalid JSON response or unexpected format
    ///
    /// # Example
    /// ```no_run
    /// use mcp_binance_server::binance::client::BinanceClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BinanceClient::new();
    /// let server_time = client.get_server_time().await?;
    /// println!("Binance server time: {}", server_time);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_server_time(&self) -> Result<i64, McpError> {
        let url = format!("{}/api/v3/time", self.base_url);
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            let response = self.client.get(&url).send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    // Handle 429 rate limit with exponential backoff
                    if status.as_u16() == 429 {
                        if retry_count >= max_retries {
                            return Err(McpError::RateLimitError(format!(
                                "Rate limit exceeded after {} retries. Wait 60 seconds before retrying.",
                                max_retries
                            )));
                        }

                        // Parse Retry-After header if present, otherwise use exponential backoff
                        let retry_after = resp
                            .headers()
                            .get("retry-after")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or_else(|| 2_u64.pow(retry_count)); // 1s, 2s, 4s

                        tracing::warn!(
                            "Rate limit hit (429). Retry {} of {}. Waiting {}s before retry.",
                            retry_count + 1,
                            max_retries,
                            retry_after
                        );

                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                        retry_count += 1;
                        continue;
                    }

                    // Check for other HTTP errors
                    if !status.is_success() {
                        return Err(McpError::from(resp.error_for_status().unwrap_err()));
                    }

                    // Parse successful response
                    let server_time_response: ServerTimeResponse = resp.json().await?;

                    // Validate response
                    if !server_time_response.is_valid() {
                        return Err(McpError::ParseError(format!(
                            "Invalid server time received: {}",
                            server_time_response.server_time
                        )));
                    }

                    return Ok(server_time_response.time_ms());
                }
                Err(err) => {
                    // Network errors are not retryable in this simple implementation
                    return Err(McpError::from(err));
                }
            }
        }
    }
}

impl Default for BinanceClient {
    fn default() -> Self {
        Self::new()
    }
}
