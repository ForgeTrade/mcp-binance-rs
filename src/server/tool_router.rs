//! Tool Router Implementation
//!
//! Handles routing of MCP tool calls to appropriate handlers using rmcp macros.
//! Automatically generates JSON Schema for tool parameters and provides
//! structured routing for all Binance API tools.

use crate::server::BinanceServer;
use rmcp::model::{CallToolResult, Content};
use rmcp::{ErrorData, tool, tool_router};
use serde_json::json;

#[tool_router(vis = "pub")]
impl BinanceServer {
    /// Get current Binance server time
    ///
    /// Returns the current server time in milliseconds since Unix epoch.
    /// Useful for time synchronization and validating server connectivity.
    ///
    /// # Returns
    /// JSON object with:
    /// - `serverTime`: Server timestamp in milliseconds
    /// - `offset`: Time difference between server and local time
    #[tool(
        description = "Returns current Binance server time in milliseconds since Unix epoch. Useful for time synchronization and connectivity validation."
    )]
    pub async fn get_server_time(&self) -> Result<CallToolResult, ErrorData> {
        // Get local time before API call
        let local_time_before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ErrorData::internal_error(format!("System time error: {}", e), None))?
            .as_millis() as i64;

        // Call Binance API
        let server_time = self
            .binance_client
            .get_server_time()
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        // Calculate offset
        let local_time_after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ErrorData::internal_error(format!("System time error: {}", e), None))?
            .as_millis() as i64;

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

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }
}
