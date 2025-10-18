//! Order flow analysis - bid/ask pressure calculation (FR-001 to FR-006)
//!
//! Tracks order flow direction by analyzing bid/ask update rates from historical snapshots.
//! Target window: 60 seconds (60 snapshots at 1/sec capture rate).

use super::{
    storage::{query::query_snapshots_in_window, SnapshotStorage},
    types::{FlowDirection, OrderFlowSnapshot},
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

/// Calculate order flow metrics for a symbol over a time window (FR-001)
///
/// Analyzes historical orderbook snapshots to determine bid/ask pressure:
/// 1. Query snapshots from RocksDB (<200ms target)
/// 2. Count bid/ask level changes (additions + cancellations)
/// 3. Calculate flow rates (updates/second)
/// 4. Determine flow direction based on bid/ask ratio
/// 5. Track cumulative delta over window
///
/// # Parameters
/// - `storage`: RocksDB snapshot storage
/// - `symbol`: Trading pair (e.g., "BTCUSDT")
/// - `window_duration_secs`: Analysis window (default: 60 seconds)
/// - `end_time`: Window end timestamp (default: now)
///
/// # Example
/// ```no_run
/// # use mcp_binance_server::orderbook::analytics::{flow::*, storage::*};
/// # async fn example(storage: SnapshotStorage) -> anyhow::Result<()> {
/// let flow = calculate_order_flow(&storage, "BTCUSDT", 60, None).await?;
/// println!("Flow direction: {:?}", flow.flow_direction);
/// println!("Bid flow: {:.2} orders/sec", flow.bid_flow_rate);
/// # Ok(())
/// # }
/// ```
pub async fn calculate_order_flow(
    storage: &SnapshotStorage,
    symbol: &str,
    window_duration_secs: u32,
    end_time: Option<DateTime<Utc>>,
) -> Result<OrderFlowSnapshot> {
    let end = end_time.unwrap_or_else(Utc::now);
    let start = end - chrono::Duration::seconds(window_duration_secs as i64);

    let start_timestamp_sec = start.timestamp();
    let end_timestamp_sec = end.timestamp();

    // Step 1: Query historical snapshots (<200ms target)
    let snapshots =
        query_snapshots_in_window(storage, symbol, start_timestamp_sec, end_timestamp_sec)
            .await
            .context("Failed to query snapshots for order flow")?;

    if snapshots.is_empty() {
        // Return neutral flow if no data available
        return Ok(OrderFlowSnapshot {
            symbol: symbol.to_string(),
            time_window_start: start,
            time_window_end: end,
            window_duration_secs,
            bid_flow_rate: 0.0,
            ask_flow_rate: 0.0,
            net_flow: 0.0,
            flow_direction: FlowDirection::Neutral,
            cumulative_delta: 0.0,
        });
    }

    // Step 2: Aggregate bid/ask counts across snapshots
    let (bid_updates, ask_updates) = aggregate_bid_ask_counts(&snapshots);

    // Step 3: Calculate flow rates (updates per second)
    let (bid_flow_rate, ask_flow_rate) =
        calculate_flow_rates(bid_updates, ask_updates, window_duration_secs);

    // Step 4: Determine flow direction based on bid/ask ratio
    let flow_direction = determine_flow_direction(bid_flow_rate, ask_flow_rate);

    // Step 5: Calculate cumulative delta
    let cumulative_delta = calculate_cumulative_delta(&snapshots);

    Ok(OrderFlowSnapshot {
        symbol: symbol.to_string(),
        time_window_start: start,
        time_window_end: end,
        window_duration_secs,
        bid_flow_rate,
        ask_flow_rate,
        net_flow: bid_flow_rate - ask_flow_rate,
        flow_direction,
        cumulative_delta,
    })
}

/// Count total bid/ask level changes across snapshots (T018)
///
/// Simplified implementation: counts non-empty levels per snapshot.
/// Production version would compare consecutive snapshots to detect:
/// - New orders (level additions)
/// - Cancellations (level removals)
/// - Quantity changes (level modifications)
fn aggregate_bid_ask_counts(snapshots: &[super::storage::snapshot::OrderBookSnapshot]) -> (usize, usize) {
    let mut bid_updates = 0;
    let mut ask_updates = 0;

    for snapshot in snapshots {
        bid_updates += snapshot.bids.len();
        ask_updates += snapshot.asks.len();
    }

    (bid_updates, ask_updates)
}

/// Calculate flow rates in updates per second (T019)
///
/// Formula: flow_rate = total_updates / window_duration
fn calculate_flow_rates(bid_updates: usize, ask_updates: usize, window_duration_secs: u32) -> (f64, f64) {
    let duration = window_duration_secs.max(1) as f64; // Avoid division by zero

    let bid_flow_rate = bid_updates as f64 / duration;
    let ask_flow_rate = ask_updates as f64 / duration;

    (bid_flow_rate, ask_flow_rate)
}

/// Determine flow direction from bid/ask flow rates (T020)
///
/// Uses FlowDirection::from_flow_rates() for classification:
/// - StrongBuy: bid_flow > 2.0 × ask_flow
/// - ModerateBuy: bid_flow 1.2-2.0 × ask_flow
/// - Neutral: bid_flow ≈ ask_flow (0.8-1.2 ratio)
/// - ModerateSell: ask_flow 1.2-2.0 × bid_flow
/// - StrongSell: ask_flow > 2.0 × bid_flow
fn determine_flow_direction(bid_flow_rate: f64, ask_flow_rate: f64) -> FlowDirection {
    if ask_flow_rate == 0.0 && bid_flow_rate > 0.0 {
        return FlowDirection::StrongBuy;
    }
    if bid_flow_rate == 0.0 && ask_flow_rate > 0.0 {
        return FlowDirection::StrongSell;
    }
    if bid_flow_rate == 0.0 && ask_flow_rate == 0.0 {
        return FlowDirection::Neutral;
    }

    FlowDirection::from_flow_rates(bid_flow_rate, ask_flow_rate)
}

/// Calculate cumulative delta over window (T021)
///
/// Simplified implementation: sums bid minus ask quantities.
/// Production version would track actual trade directions from @aggTrade stream.
fn calculate_cumulative_delta(snapshots: &[super::storage::snapshot::OrderBookSnapshot]) -> f64 {
    let mut cumulative_delta = 0.0;

    for snapshot in snapshots {
        // Sum bid quantities (buying pressure)
        let bid_qty: f64 = snapshot
            .bids
            .iter()
            .filter_map(|(_, qty)| qty.parse::<f64>().ok())
            .sum();

        // Sum ask quantities (selling pressure)
        let ask_qty: f64 = snapshot
            .asks
            .iter()
            .filter_map(|(_, qty)| qty.parse::<f64>().ok())
            .sum();

        cumulative_delta += bid_qty - ask_qty;
    }

    cumulative_delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_bid_ask_counts() {
        use super::super::storage::snapshot::OrderBookSnapshot;

        let snapshots = vec![
            OrderBookSnapshot {
                bids: vec![("100.0".to_string(), "1.0".to_string())],
                asks: vec![("101.0".to_string(), "1.0".to_string())],
                update_id: 1,
                timestamp: 1000,
            },
            OrderBookSnapshot {
                bids: vec![
                    ("100.0".to_string(), "1.0".to_string()),
                    ("99.9".to_string(), "0.5".to_string()),
                ],
                asks: vec![("101.0".to_string(), "1.0".to_string())],
                update_id: 2,
                timestamp: 1001,
            },
        ];

        let (bid_updates, ask_updates) = aggregate_bid_ask_counts(&snapshots);
        assert_eq!(bid_updates, 3); // 1 + 2
        assert_eq!(ask_updates, 2); // 1 + 1
    }

    #[test]
    fn test_calculate_flow_rates() {
        let (bid_rate, ask_rate) = calculate_flow_rates(120, 60, 60);
        assert_eq!(bid_rate, 2.0); // 120 updates / 60 seconds
        assert_eq!(ask_rate, 1.0); // 60 updates / 60 seconds
    }

    #[test]
    fn test_determine_flow_direction() {
        assert_eq!(
            determine_flow_direction(10.0, 4.0),
            FlowDirection::StrongBuy
        ); // Ratio 2.5
        assert_eq!(
            determine_flow_direction(6.0, 4.0),
            FlowDirection::ModerateBuy
        ); // Ratio 1.5
        assert_eq!(
            determine_flow_direction(5.0, 5.0),
            FlowDirection::Neutral
        ); // Ratio 1.0
        assert_eq!(
            determine_flow_direction(4.0, 6.0),
            FlowDirection::ModerateSell
        ); // Ratio 0.67
        assert_eq!(
            determine_flow_direction(2.0, 10.0),
            FlowDirection::StrongSell
        ); // Ratio 0.2
    }

    #[test]
    fn test_calculate_cumulative_delta() {
        use super::super::storage::snapshot::OrderBookSnapshot;

        let snapshots = vec![
            OrderBookSnapshot {
                bids: vec![("100.0".to_string(), "2.0".to_string())],
                asks: vec![("101.0".to_string(), "1.0".to_string())],
                update_id: 1,
                timestamp: 1000,
            },
            OrderBookSnapshot {
                bids: vec![("100.0".to_string(), "3.0".to_string())],
                asks: vec![("101.0".to_string(), "2.0".to_string())],
                update_id: 2,
                timestamp: 1001,
            },
        ];

        let delta = calculate_cumulative_delta(&snapshots);
        // (2.0 - 1.0) + (3.0 - 2.0) = 1.0 + 1.0 = 2.0
        assert_eq!(delta, 2.0);
    }
}
