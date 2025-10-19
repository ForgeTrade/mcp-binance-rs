//! MCP tool implementations for advanced orderbook analytics
//!
//! This module defines MCP tools for accessing order flow, volume profile,
//! and anomaly detection features.

use super::{
    anomaly::detect_anomalies, flow::calculate_order_flow, health::calculate_health_score,
    profile::generate_volume_profile, storage::SnapshotStorage, types::LiquidityVacuum,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{tool, ErrorData};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

/// Input parameters for get_order_flow tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetOrderFlowInput {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Analysis window duration in seconds (default: 60)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_duration_secs: Option<u32>,
}

/// Input parameters for get_volume_profile tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetVolumeProfileInput {
    /// Trading pair symbol (e.g., "ETHUSDT")
    pub symbol: String,

    /// Analysis period in hours (default: 24)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_hours: Option<u32>,

    /// Price tick size for binning (e.g., "0.01")
    pub tick_size: String,
}

/// Input parameters for detect_market_anomalies tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectMarketAnomaliesInput {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Analysis window duration in seconds (default: 60)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_duration_secs: Option<u32>,
}

/// Input parameters for get_liquidity_vacuums tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetLiquidityVacuumsInput {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Analysis period in hours (default: 24)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_hours: Option<u32>,

    /// Price tick size for binning (e.g., "0.01")
    pub tick_size: String,
}

/// Input parameters for get_microstructure_health tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetMicrostructureHealthInput {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Analysis window duration in seconds (default: 300)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_duration_secs: Option<u32>,
}

/// Get Order Flow Analysis (T022, FR-001 to FR-006)
///
/// Analyzes bid/ask pressure and flow direction over a time window.
/// Returns flow rates, net flow, direction classification, and cumulative delta.
#[tool(
    description = "Analyze order flow direction and bid/ask pressure over time window. Returns flow rates, net flow, direction classification, and cumulative delta."
)]
pub async fn get_order_flow(
    params: Parameters<GetOrderFlowInput>,
    storage: Arc<SnapshotStorage>,
) -> Result<CallToolResult, ErrorData> {
    let window_duration = params.0.window_duration_secs.unwrap_or(60);

    let flow_snapshot = calculate_order_flow(&storage, &params.0.symbol, window_duration, None)
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

    let response_json = serde_json::to_value(&flow_snapshot)
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response_json.to_string(),
    )]))
}

/// Get Volume Profile (T032, FR-007 to FR-008)
///
/// Generates volume distribution histogram with POC/VAH/VAL for support/resistance identification.
/// Returns POC (Point of Control), VAH/VAL (Value Area High/Low) for identifying support/resistance.
#[tool(
    description = "Generate volume profile histogram showing volume distribution across price levels. Returns POC (Point of Control), VAH/VAL (Value Area High/Low) for support/resistance identification."
)]
pub async fn get_volume_profile(
    params: Parameters<GetVolumeProfileInput>,
) -> Result<CallToolResult, ErrorData> {
    let duration = params.0.duration_hours.unwrap_or(24);
    let tick = Decimal::from_str_exact(&params.0.tick_size)
        .map_err(|e| ErrorData::invalid_params(format!("Invalid tick_size format: {}", e), None))?;

    let volume_profile = generate_volume_profile(&params.0.symbol, duration, tick)
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

    let response_json = serde_json::to_value(&volume_profile)
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response_json.to_string(),
    )]))
}

/// Detect Market Anomalies (T040, FR-003 to FR-005)
///
/// Detects market microstructure anomalies including quote stuffing (HFT manipulation),
/// iceberg orders (hidden institutional orders), and flash crash risk (extreme liquidity deterioration).
/// Returns anomalies with severity levels and recommended actions.
#[tool(
    description = "Detect market microstructure anomalies including quote stuffing (HFT manipulation), iceberg orders (hidden institutional orders), and flash crash risk (extreme liquidity deterioration). Returns anomalies with severity levels and recommended actions."
)]
pub async fn detect_market_anomalies(
    params: Parameters<DetectMarketAnomaliesInput>,
    storage: Arc<SnapshotStorage>,
) -> Result<CallToolResult, ErrorData> {
    let window_duration = params.0.window_duration_secs.unwrap_or(60);

    let anomalies = detect_anomalies(&storage, &params.0.symbol, window_duration)
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

    let response_json = serde_json::to_value(&anomalies)
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response_json.to_string(),
    )]))
}

/// Get Liquidity Vacuums (T041, FR-008)
///
/// Identifies price ranges with abnormally low volume (<20% median). These zones are prone to
/// fast price movements when crossed. Returns vacuum locations with expected impact levels.
#[tool(
    description = "Identify liquidity vacuums - price ranges with abnormally low volume (<20% median). These zones are prone to fast price movements when crossed. Returns vacuum locations with expected impact levels."
)]
pub async fn get_liquidity_vacuums(
    params: Parameters<GetLiquidityVacuumsInput>,
) -> Result<CallToolResult, ErrorData> {
    let duration = params.0.duration_hours.unwrap_or(24);
    let tick = Decimal::from_str_exact(&params.0.tick_size)
        .map_err(|e| ErrorData::invalid_params(format!("Invalid tick_size format: {}", e), None))?;

    // Generate volume profile first
    let profile = generate_volume_profile(&params.0.symbol, duration, tick)
        .await
        .map_err(|e| {
            ErrorData::internal_error(format!("Failed to generate volume profile: {}", e), None)
        })?;

    // Calculate median volume
    let median_volume = if profile.histogram.is_empty() {
        let response_json = serde_json::json!([]);
        return Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]));
    } else {
        let mut volumes: Vec<Decimal> = profile.histogram.iter().map(|b| b.volume).collect();
        volumes.sort();
        volumes[volumes.len() / 2]
    };

    let vacuum_threshold = median_volume
        * Decimal::from_str("0.20").map_err(|e| {
            ErrorData::internal_error(format!("Decimal conversion error: {}", e), None)
        })?;

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

            let volume_deficit_pct_decimal = ((median_volume - avg_volume_in_range)
                / median_volume)
                * Decimal::from_str("100.0").map_err(|e| {
                    ErrorData::internal_error(format!("Decimal conversion error: {}", e), None)
                })?;

            let volume_deficit_pct = volume_deficit_pct_decimal
                .to_string()
                .parse::<f64>()
                .map_err(|e| ErrorData::internal_error(format!("Parse error: {}", e), None))?;

            let expected_impact = if volume_deficit_pct > 80.0 {
                super::types::ImpactLevel::FastMovement
            } else if volume_deficit_pct > 50.0 {
                super::types::ImpactLevel::ModerateMovement
            } else {
                super::types::ImpactLevel::Negligible
            };

            vacuums.push(LiquidityVacuum {
                vacuum_id: uuid::Uuid::new_v4(),
                symbol: params.0.symbol.clone(),
                price_range_low,
                price_range_high,
                volume_deficit_pct,
                median_volume,
                actual_volume: avg_volume_in_range,
                expected_impact,
                detection_timestamp: chrono::Utc::now(),
            });

            vacuum_start = None;
        }
    }

    let response_json = serde_json::to_value(&vacuums)
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response_json.to_string(),
    )]))
}

/// Get Microstructure Health Score (T042, FR-010)
///
/// Calculates composite 0-100 market health score combining spread stability, liquidity depth,
/// flow balance, and update rate. Returns overall score, component breakdown, health level,
/// and recommended actions.
#[tool(
    description = "Calculate market microstructure health score (0-100) combining spread stability, liquidity depth, flow balance, and update rate. Returns overall score, component breakdown, health level, and recommended actions."
)]
pub async fn get_microstructure_health(
    params: Parameters<GetMicrostructureHealthInput>,
    storage: Arc<SnapshotStorage>,
) -> Result<CallToolResult, ErrorData> {
    let window_duration = params.0.window_duration_secs.unwrap_or(300);

    let health = calculate_health_score(&storage, &params.0.symbol, window_duration)
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

    let response_json = serde_json::to_value(&health)
        .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response_json.to_string(),
    )]))
}
