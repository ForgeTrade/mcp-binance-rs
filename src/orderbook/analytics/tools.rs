//! MCP tool implementations for advanced orderbook analytics
//!
//! This module defines MCP tools for accessing order flow, volume profile,
//! and anomaly detection features.

use super::{
    anomaly::detect_anomalies,
    flow::calculate_order_flow,
    health::calculate_health_score,
    profile::generate_volume_profile,
    storage::SnapshotStorage,
    types::{LiquidityVacuum, MarketMicrostructureAnomaly, MicrostructureHealth, OrderFlowSnapshot, VolumeProfile},
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

/// MCP Tool: Detect Market Anomalies (T040, FR-003 to FR-005)
///
/// Detects market microstructure anomalies including HFT manipulation patterns.
///
/// # Arguments
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `window_duration_secs`: Analysis window in seconds (default: 60)
///
/// # Returns
/// Array of detected anomalies with:
/// - anomaly_type: QuoteStuffing | IcebergOrder | FlashCrashRisk
/// - severity: Low | Medium | High | Critical
/// - confidence: 0.0-1.0 detection confidence
/// - recommended_action: Trading guidance
///
/// # Detectors
/// 1. **Quote Stuffing** (FR-003): >500 updates/sec, <10% fill rate
/// 2. **Iceberg Orders** (FR-004): Refill rate >5x median
/// 3. **Flash Crash Risk** (FR-005): >80% depth loss, >10x spread, >90% cancellations
///
/// # Example Usage
/// ```json
/// {
///   "symbol": "BTCUSDT",
///   "window_duration_secs": 60
/// }
/// ```
#[tool(
    description = "Detect market microstructure anomalies including quote stuffing (HFT manipulation), iceberg orders (hidden institutional orders), and flash crash risk (extreme liquidity deterioration). Returns anomalies with severity levels and recommended actions."
)]
pub async fn detect_market_anomalies(
    #[tool(description = "Trading pair symbol (e.g., BTCUSDT)")]
    symbol: String,

    #[tool(description = "Analysis window duration in seconds (default: 60)")]
    window_duration_secs: Option<u32>,

    #[tool(shared_state)]
    storage: Arc<SnapshotStorage>,
) -> Result<Vec<MarketMicrostructureAnomaly>> {
    let window_duration = window_duration_secs.unwrap_or(60);

    detect_anomalies(&storage, &symbol, window_duration)
        .await
        .context("Failed to detect market anomalies")
}

/// MCP Tool: Get Liquidity Vacuums (T041, FR-008)
///
/// Identifies price ranges with abnormally low volume (<20% of median).
///
/// # Arguments
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `duration_hours`: Analysis period in hours (default: 24)
/// - `tick_size`: Price tick size (e.g., "0.01")
///
/// # Returns
/// Array of liquidity vacuums with:
/// - price_range_low/high: Vacuum boundaries
/// - volume_deficit_pct: How much below median volume
/// - expected_impact: Price movement risk (FastMovement | ModerateMovement | Negligible)
///
/// # Example Usage
/// ```json
/// {
///   "symbol": "BTCUSDT",
///   "duration_hours": 24,
///   "tick_size": "0.01"
/// }
/// ```
#[tool(
    description = "Identify liquidity vacuums - price ranges with abnormally low volume (<20% median). These zones are prone to fast price movements when crossed. Returns vacuum locations with expected impact levels."
)]
pub async fn get_liquidity_vacuums(
    #[tool(description = "Trading pair symbol (e.g., BTCUSDT)")]
    symbol: String,

    #[tool(description = "Analysis period in hours (default: 24)")]
    duration_hours: Option<u32>,

    #[tool(description = "Price tick size for binning (e.g., 0.01)")]
    tick_size: String,
) -> Result<Vec<LiquidityVacuum>> {
    let duration = duration_hours.unwrap_or(24);
    let tick = Decimal::from_str_exact(&tick_size)
        .context("Invalid tick_size format")?;

    // Generate volume profile first
    let profile = generate_volume_profile(&symbol, duration, tick)
        .await
        .context("Failed to generate volume profile for vacuum detection")?;

    // Calculate median volume
    let median_volume = if profile.histogram.is_empty() {
        return Ok(Vec::new());
    } else {
        let mut volumes: Vec<Decimal> = profile.histogram.iter().map(|b| b.volume).collect();
        volumes.sort();
        volumes[volumes.len() / 2]
    };

    let vacuum_threshold = median_volume * Decimal::from_str("0.20")?; // <20% of median

    // Identify vacuums
    let mut vacuums = Vec::new();
    let mut vacuum_start: Option<usize> = None;

    for (idx, bin) in profile.histogram.iter().enumerate() {
        if bin.volume < vacuum_threshold {
            if vacuum_start.is_none() {
                vacuum_start = Some(idx);
            }
        } else if let Some(start_idx) = vacuum_start {
            // Vacuum ended, create entry
            let price_range_low = profile.histogram[start_idx].price_level;
            let price_range_high = bin.price_level;

            let avg_volume_in_range = profile.histogram[start_idx..idx]
                .iter()
                .map(|b| b.volume)
                .sum::<Decimal>()
                / Decimal::from(idx - start_idx);

            let volume_deficit_pct = ((median_volume - avg_volume_in_range) / median_volume)
                * Decimal::from_str("100.0")?;

            let expected_impact = if volume_deficit_pct > Decimal::from_str("80.0")? {
                super::types::ImpactLevel::FastMovement
            } else if volume_deficit_pct > Decimal::from_str("50.0")? {
                super::types::ImpactLevel::ModerateMovement
            } else {
                super::types::ImpactLevel::Negligible
            };

            vacuums.push(LiquidityVacuum {
                vacuum_id: uuid::Uuid::new_v4(),
                symbol: symbol.clone(),
                price_range_low,
                price_range_high,
                volume_deficit_pct,
                expected_impact,
                detected_at: chrono::Utc::now(),
            });

            vacuum_start = None;
        }
    }

    Ok(vacuums)
}

/// MCP Tool: Get Microstructure Health Score (T042, FR-010)
///
/// Calculates composite 0-100 market health score.
///
/// # Arguments
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `window_duration_secs`: Analysis window in seconds (default: 300)
///
/// # Returns
/// MicrostructureHealth with:
/// - overall_score: 0-100 composite score
/// - Component scores: spread_stability, liquidity_depth, flow_balance, update_rate
/// - health_level: Excellent | Good | Fair | Poor | Critical
/// - recommended_action: Trading guidance
///
/// # Scoring
/// - 80-100: Excellent (safe to trade aggressively)
/// - 60-79: Good (normal conditions)
/// - 40-59: Fair (exercise caution)
/// - 20-39: Poor (reduce positions)
/// - 0-19: Critical (halt trading)
///
/// # Example Usage
/// ```json
/// {
///   "symbol": "BTCUSDT",
///   "window_duration_secs": 300
/// }
/// ```
#[tool(
    description = "Calculate market microstructure health score (0-100) combining spread stability, liquidity depth, flow balance, and update rate. Returns overall score, component breakdown, health level, and recommended actions."
)]
pub async fn get_microstructure_health(
    #[tool(description = "Trading pair symbol (e.g., BTCUSDT)")]
    symbol: String,

    #[tool(description = "Analysis window duration in seconds (default: 300)")]
    window_duration_secs: Option<u32>,

    #[tool(shared_state)]
    storage: Arc<SnapshotStorage>,
) -> Result<MicrostructureHealth> {
    let window_duration = window_duration_secs.unwrap_or(300);

    calculate_health_score(&storage, &symbol, window_duration)
        .await
        .context("Failed to calculate microstructure health")
}
