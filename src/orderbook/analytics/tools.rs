//! MCP tool implementations for advanced orderbook analytics
//!
//! This module defines MCP tools for accessing order flow, volume profile,
//! and anomaly detection features.

use super::{
    flow::calculate_order_flow,
    storage::SnapshotStorage,
    types::OrderFlowSnapshot,
};
use anyhow::{Context, Result};
use rmcp::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Input parameters for get_order_flow tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetOrderFlowInput {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Analysis window duration in seconds (default: 60)
    #[serde(default = "default_window_duration")]
    pub window_duration_secs: u32,
}

fn default_window_duration() -> u32 {
    60
}

/// MCP Tool: Get Order Flow Analysis (T022, FR-001 to FR-006)
///
/// Calculates bid/ask pressure and flow direction over a time window.
///
/// # Arguments
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `window_duration_secs`: Analysis window in seconds (default: 60)
///
/// # Returns
/// OrderFlowSnapshot with:
/// - bid_flow_rate: Bid updates per second
/// - ask_flow_rate: Ask updates per second
/// - net_flow: Bid flow - ask flow
/// - flow_direction: StrongBuy | ModerateBuy | Neutral | ModerateSell | StrongSell
/// - cumulative_delta: Cumulative bid-ask quantity difference
///
/// # Example Usage
/// ```json
/// {
///   "symbol": "BTCUSDT",
///   "window_duration_secs": 60
/// }
/// ```
#[tool(
    description = "Analyze order flow direction and bid/ask pressure over time window. Returns flow rates, net flow, direction classification, and cumulative delta."
)]
pub async fn get_order_flow(
    #[tool(description = "Trading pair symbol (e.g., BTCUSDT)")]
    symbol: String,

    #[tool(description = "Analysis window duration in seconds (default: 60)")]
    window_duration_secs: Option<u32>,

    #[tool(shared_state)]
    storage: Arc<SnapshotStorage>,
) -> Result<OrderFlowSnapshot> {
    let window_duration = window_duration_secs.unwrap_or(60);

    calculate_order_flow(&storage, &symbol, window_duration, None)
        .await
        .context("Failed to calculate order flow")
}

// TODO: T032 - get_volume_profile tool
// TODO: T040 - detect_anomalies tool
// TODO: T041 - get_liquidity_vacuums tool
// TODO: T042 - get_health_score tool
