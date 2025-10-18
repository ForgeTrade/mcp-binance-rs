//! Market microstructure health scoring (FR-010)
//!
//! Composite 0-100 health score combining:
//! - Spread stability (low volatility = healthy)
//! - Liquidity depth (thick orderbook = healthy)
//! - Flow balance (neutral flow = healthy)
//! - Update rate (moderate activity = healthy)

use super::{
    storage::{SnapshotStorage, query::query_snapshots_in_window},
    types::MicrostructureHealth,
};
use anyhow::{Context, Result};
use chrono::Utc;

/// Calculate market microstructure health score (T042, FR-010)
///
/// Generates composite 0-100 score from four components:
/// 1. **Spread Stability** (25%): Measures bid-ask spread volatility
/// 2. **Liquidity Depth** (35%): Total orderbook depth across price levels
/// 3. **Flow Balance** (25%): Bid/ask flow equilibrium (neutral is healthiest)
/// 4. **Update Rate** (15%): Market activity level (moderate is optimal)
///
/// # Scoring
/// - 80-100: Excellent (safe to trade aggressively)
/// - 60-79: Good (normal trading conditions)
/// - 40-59: Fair (exercise caution)
/// - 20-39: Poor (reduce position sizes)
/// - 0-19: Critical (halt trading, high risk)
///
/// # Parameters
/// - `storage`: RocksDB snapshot storage
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `window_duration_secs`: Analysis window (default: 300 seconds / 5 minutes)
///
/// # Example
/// ```no_run
/// # use mcp_binance_server::orderbook::analytics::{health::*, storage::*};
/// # async fn example(storage: SnapshotStorage) -> anyhow::Result<()> {
/// let health = calculate_health_score(&storage, "BTCUSDT", 300).await?;
/// println!("Health score: {} ({})", health.overall_score, health.health_level);
/// # Ok(())
/// # }
/// ```
pub async fn calculate_health_score(
    storage: &SnapshotStorage,
    symbol: &str,
    window_duration_secs: u32,
) -> Result<MicrostructureHealth> {
    let end = Utc::now();
    let start = end - chrono::Duration::seconds(window_duration_secs as i64);

    // Query snapshots
    let snapshots = query_snapshots_in_window(storage, symbol, start.timestamp(), end.timestamp())
        .await
        .context("Failed to query snapshots for health score")?;

    if snapshots.is_empty() {
        // Return critical health if no data
        return Ok(MicrostructureHealth {
            symbol: symbol.to_string(),
            timestamp: end,
            overall_score: 0.0,
            spread_stability_score: 0.0,
            liquidity_depth_score: 0.0,
            flow_balance_score: 0.0,
            update_rate_score: 0.0,
            health_level: "Critical".to_string(),
            recommended_action: "No market data available. HALT TRADING.".to_string(),
        });
    }

    // Calculate component scores
    let spread_stability = calculate_spread_stability_score(&snapshots);
    let liquidity_depth = calculate_liquidity_depth_score(&snapshots);
    let flow_balance = calculate_flow_balance_score(&snapshots);
    let update_rate = calculate_update_rate_score(&snapshots, window_duration_secs);

    // Composite score with weighted components
    let overall_score = (spread_stability * 0.25)
        + (liquidity_depth * 0.35)
        + (flow_balance * 0.25)
        + (update_rate * 0.15);

    let (health_level, recommended_action) = classify_health(overall_score);

    Ok(MicrostructureHealth {
        symbol: symbol.to_string(),
        timestamp: end,
        overall_score,
        spread_stability_score: spread_stability,
        liquidity_depth_score: liquidity_depth,
        flow_balance_score: flow_balance,
        update_rate_score: update_rate,
        health_level,
        recommended_action,
    })
}

/// Calculate spread stability score (0-100)
///
/// Measures bid-ask spread volatility. Lower volatility = higher score.
fn calculate_spread_stability_score(
    snapshots: &[super::storage::snapshot::OrderBookSnapshot],
) -> f64 {
    if snapshots.len() < 2 {
        return 50.0; // Neutral if insufficient data
    }

    let spreads: Vec<f64> = snapshots
        .iter()
        .filter_map(|snap| {
            if snap.bids.is_empty() || snap.asks.is_empty() {
                return None;
            }
            let bid = snap.bids[0].0.parse::<f64>().ok()?;
            let ask = snap.asks[0].0.parse::<f64>().ok()?;
            Some(ask - bid)
        })
        .collect();

    if spreads.is_empty() {
        return 0.0;
    }

    // Calculate coefficient of variation (CV = std_dev / mean)
    let mean = spreads.iter().sum::<f64>() / spreads.len() as f64;
    let variance = spreads.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / spreads.len() as f64;
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 1.0 };

    // Convert CV to 0-100 score (lower CV = higher score)
    // CV < 0.05 = 100, CV > 0.5 = 0
    let score = 100.0 * (1.0 - (cv / 0.5).min(1.0));
    score.clamp(0.0, 100.0)
}

/// Calculate liquidity depth score (0-100)
///
/// Measures total orderbook thickness. More levels = higher score.
fn calculate_liquidity_depth_score(
    snapshots: &[super::storage::snapshot::OrderBookSnapshot],
) -> f64 {
    if snapshots.is_empty() {
        return 0.0;
    }

    // Average depth across snapshots
    let avg_depth = snapshots
        .iter()
        .map(|snap| (snap.bids.len() + snap.asks.len()) as f64)
        .sum::<f64>()
        / snapshots.len() as f64;

    // Normalize to 0-100 (assume 100+ levels = perfect depth)
    let score = (avg_depth / 100.0) * 100.0;
    score.clamp(0.0, 100.0)
}

/// Calculate flow balance score (0-100)
///
/// Measures bid/ask flow equilibrium. Neutral flow (ratio ≈ 1.0) = higher score.
fn calculate_flow_balance_score(snapshots: &[super::storage::snapshot::OrderBookSnapshot]) -> f64 {
    if snapshots.is_empty() {
        return 50.0; // Neutral
    }

    let total_bid_levels: usize = snapshots.iter().map(|s| s.bids.len()).sum();
    let total_ask_levels: usize = snapshots.iter().map(|s| s.asks.len()).sum();

    if total_bid_levels == 0 && total_ask_levels == 0 {
        return 0.0;
    }

    let ratio = if total_ask_levels > 0 {
        total_bid_levels as f64 / total_ask_levels as f64
    } else {
        2.0 // Imbalanced
    };

    // Perfect balance = 1.0, score = 100
    // Strong imbalance (ratio <0.5 or >2.0) = 0
    let score = if (0.8..=1.2).contains(&ratio) {
        100.0 // Excellent balance
    } else if (0.5..=2.0).contains(&ratio) {
        50.0 + (50.0 * (1.0 - ((ratio - 1.0).abs() / 1.0)))
    } else {
        0.0 // Severe imbalance
    };

    score.clamp(0.0, 100.0)
}

/// Calculate update rate score (0-100)
///
/// Measures market activity level. Moderate activity (10-100 updates/sec) = higher score.
fn calculate_update_rate_score(
    snapshots: &[super::storage::snapshot::OrderBookSnapshot],
    window_duration_secs: u32,
) -> f64 {
    let update_count = snapshots.len() as f64;
    let duration = window_duration_secs.max(1) as f64;
    let update_rate = update_count / duration;

    // Optimal: 10-100 updates/sec = 100 score
    // Too slow (<1/sec) or too fast (>500/sec) = low score
    let score = if (10.0..=100.0).contains(&update_rate) {
        100.0
    } else if update_rate < 1.0 {
        update_rate * 50.0 // Linear scale 0-1 -> 0-50
    } else if update_rate < 10.0 {
        50.0 + ((update_rate - 1.0) / 9.0) * 50.0 // 1-10 -> 50-100
    } else if update_rate <= 500.0 {
        100.0 - ((update_rate - 100.0) / 400.0) * 50.0 // 100-500 -> 100-50
    } else {
        0.0 // Too fast (likely quote stuffing)
    };

    score.clamp(0.0, 100.0)
}

/// Classify health score into levels
fn classify_health(score: f64) -> (String, String) {
    if score >= 80.0 {
        (
            "Excellent".to_string(),
            "Market conditions are optimal. Safe to trade aggressively with normal position sizes."
                .to_string(),
        )
    } else if score >= 60.0 {
        (
            "Good".to_string(),
            "Normal trading conditions. Standard risk management applies.".to_string(),
        )
    } else if score >= 40.0 {
        (
            "Fair".to_string(),
            "Exercise caution. Consider tighter stops and smaller position sizes.".to_string(),
        )
    } else if score >= 20.0 {
        (
            "Poor".to_string(),
            "Market conditions deteriorating. Reduce position sizes by 50% and widen stops."
                .to_string(),
        )
    } else {
        (
            "Critical".to_string(),
            "SEVERE RISK. Halt new trades immediately. Exit positions or hedge exposures."
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::storage::snapshot::OrderBookSnapshot;
    use super::*;

    #[test]
    fn test_calculate_spread_stability_score() {
        // Create snapshots with stable spread (1.0)
        let stable_snapshots: Vec<OrderBookSnapshot> = (0..10)
            .map(|i| OrderBookSnapshot {
                bids: vec![("100.0".to_string(), "10.0".to_string())],
                asks: vec![("101.0".to_string(), "10.0".to_string())],
                update_id: i,
                timestamp: 1000 + i as i64,
            })
            .collect();

        let score = calculate_spread_stability_score(&stable_snapshots);
        assert!(score > 90.0); // Stable spread = high score
    }

    #[test]
    fn test_calculate_liquidity_depth_score() {
        // Create thick orderbook (100 levels total)
        let thick_snapshot = OrderBookSnapshot {
            bids: vec![("100.0".to_string(), "10.0".to_string()); 50],
            asks: vec![("101.0".to_string(), "10.0".to_string()); 50],
            update_id: 1,
            timestamp: 1000,
        };

        let score = calculate_liquidity_depth_score(&[thick_snapshot]);
        assert!(score >= 95.0); // 100 levels = near perfect score
    }

    #[test]
    fn test_calculate_flow_balance_score() {
        // Create perfectly balanced snapshot
        let balanced = OrderBookSnapshot {
            bids: vec![("100.0".to_string(), "10.0".to_string()); 10],
            asks: vec![("101.0".to_string(), "10.0".to_string()); 10],
            update_id: 1,
            timestamp: 1000,
        };

        let score = calculate_flow_balance_score(&[balanced]);
        assert_eq!(score, 100.0); // Perfect balance = 100
    }

    #[test]
    fn test_calculate_update_rate_score() {
        // Create 60 snapshots over 1 second (60 updates/sec = optimal)
        let snapshots: Vec<OrderBookSnapshot> = (0..60)
            .map(|i| OrderBookSnapshot {
                bids: vec![("100.0".to_string(), "1.0".to_string())],
                asks: vec![("101.0".to_string(), "1.0".to_string())],
                update_id: i,
                timestamp: 1000 + i as i64,
            })
            .collect();

        let score = calculate_update_rate_score(&snapshots, 1);
        assert_eq!(score, 100.0); // 60/sec is optimal range
    }

    #[test]
    fn test_classify_health() {
        let (level, _) = classify_health(90.0);
        assert_eq!(level, "Excellent");

        let (level, _) = classify_health(70.0);
        assert_eq!(level, "Good");

        let (level, _) = classify_health(50.0);
        assert_eq!(level, "Fair");

        let (level, _) = classify_health(30.0);
        assert_eq!(level, "Poor");

        let (level, _) = classify_health(10.0);
        assert_eq!(level, "Critical");
    }
}
