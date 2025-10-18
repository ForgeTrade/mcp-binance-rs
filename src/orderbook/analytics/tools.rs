//! MCP tool implementations for advanced orderbook analytics
//!
//! This module defines MCP tools for accessing order flow, volume profile,
//! and anomaly detection features.

use super::{
    flow::calculate_order_flow,
    profile::generate_volume_profile,
    storage::SnapshotStorage,
    types::{OrderFlowSnapshot, VolumeProfile},
};
use rust_decimal::Decimal;
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

/// MCP Tool: Get Volume Profile (T032, FR-007 to FR-008)
///
/// Generates volume distribution histogram with POC/VAH/VAL for support/resistance identification.
///
/// # Arguments
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `duration_hours`: Analysis period in hours (default: 24)
/// - `tick_size`: Price tick size for adaptive binning (e.g., "0.01" for BTCUSDT)
///
/// # Returns
/// VolumeProfile with:
/// - histogram: Price bins with volume and trade counts
/// - poc_price: Point of Control (max volume price level)
/// - vah_price: Value Area High (70% volume upper bound)
/// - val_price: Value Area Low (70% volume lower bound)
///
/// # Example Usage
/// ```json
/// {
///   "symbol": "ETHUSDT",
///   "duration_hours": 24,
///   "tick_size": "0.01"
/// }
/// ```
#[tool(
    description = "Generate volume profile histogram showing volume distribution across price levels. Returns POC (Point of Control), VAH/VAL (Value Area High/Low) for support/resistance identification."
)]
pub async fn get_volume_profile(
    #[tool(description = "Trading pair symbol (e.g., ETHUSDT)")]
    symbol: String,

    #[tool(description = "Analysis period in hours (default: 24)")]
    duration_hours: Option<u32>,

    #[tool(description = "Price tick size for binning (e.g., 0.01)")]
    tick_size: String,
) -> Result<VolumeProfile> {
    let duration = duration_hours.unwrap_or(24);
    let tick = Decimal::from_str_exact(&tick_size)
        .context("Invalid tick_size format")?;

    generate_volume_profile(&symbol, duration, tick)
        .await
        .context("Failed to generate volume profile")
}

// TODO: T040 - detect_anomalies tool
// TODO: T041 - get_liquidity_vacuums tool
// TODO: T042 - get_health_score tool
