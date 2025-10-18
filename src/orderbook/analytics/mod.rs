//! Advanced Order Book Analytics Module
//!
//! This module provides sophisticated market microstructure analysis tools including:
//! - Order flow analysis (bid/ask pressure dynamics)
//! - Volume profile generation (POC, VAH, VAL support/resistance zones)
//! - Anomaly detection (quote stuffing, iceberg orders, flash crash precursors)
//!
//! **Feature Gate**: `orderbook_analytics` (extends `orderbook` feature)
//!
//! # Example
//! ```no_run
//! # #[cfg(feature = "orderbook_analytics")]
//! # use mcp_binance_server::orderbook::analytics::*;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Get order flow metrics for BTCUSDT over 60 seconds
//! let flow = calculate_order_flow("BTCUSDT", 60).await?;
//! println!("Net flow: {} ({})", flow.net_flow, flow.flow_direction);
//! # Ok(())
//! # }
//! ```

// Re-export public types
pub mod types;
pub use types::*;

// Core analytics modules
pub mod storage;
pub mod flow;
pub mod profile;
pub mod anomaly;
pub mod health;
pub mod trade_stream;
pub mod tools;

/// Module version aligned with spec 008-orderbook-advanced-analytics
pub const VERSION: &str = "0.1.0";

/// Feature gate check - compile-time validation
#[cfg(not(feature = "orderbook"))]
compile_error!(
    "orderbook_analytics requires the 'orderbook' feature. \
     Enable with: --features orderbook_analytics"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
