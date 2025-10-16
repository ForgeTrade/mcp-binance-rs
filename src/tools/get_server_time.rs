//! Get Server Time Tool
//!
//! MCP tool for fetching current Binance server time.
//! Used for time synchronization and validating server connectivity.

use crate::error::McpError;
use crate::server::BinanceServer;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

impl BinanceServer {
    /// Returns current Binance server time
    ///
    /// MCP tool that fetches the current server time from Binance API.
    /// Useful for time synchronization and validating server connectivity.
    ///
    /// # Returns
    /// * `Ok(CallToolResult)` - Success with server time in JSON format
    /// * `Err(McpError)` - Network error, rate limit, or parse error
    ///
    /// # Response Format
    /// ```json
    /// {
    ///   "serverTime": 1699564800000,
    ///   "offset": -125
    /// }
    /// ```
    ///
    /// Where:
    /// - `serverTime`: Server time in milliseconds since Unix epoch
    /// - `offset`: Time difference between server and local time in milliseconds
    pub async fn get_server_time_tool(&self) -> Result<CallToolResult, McpError> {
        // Get local time before API call
        let local_time_before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| McpError::InternalError(format!("System time error: {}", e)))?
            .as_millis() as i64;

        // Call Binance API
        let server_time = self.binance_client.get_server_time().await?;

        // Calculate offset
        let local_time_after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| McpError::InternalError(format!("System time error: {}", e)))?
            .as_millis() as i64;

        // Use average of before/after for more accurate offset calculation
        let local_time_avg = (local_time_before + local_time_after) / 2;
        let offset = server_time - local_time_avg;

        // Log time synchronization info
        tracing::info!(
            "Binance server time: {} (offset: {}ms)",
            server_time,
            offset
        );

        // Warn if offset is significant (>5 seconds)
        if offset.abs() > 5000 {
            tracing::warn!(
                "Large time offset detected: {}ms. Consider syncing system clock.",
                offset
            );
        }

        // Create response JSON
        let response_json = json!({
            "serverTime": server_time,
            "offset": offset
        });

        Ok(CallToolResult {
            content: vec![Content::text(response_json.to_string())],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}
