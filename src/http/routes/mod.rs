//! HTTP REST API route handlers
//!
//! Provides REST endpoints for Binance API operations:
//! - Market data: prices, orderbook, trades, candles
//! - Orders: create, cancel, query
//! - Account: balance, positions

#[cfg(feature = "http-api")]
pub mod market_data;

// Order routes will be added in Phase 4 (US2)
// Account routes will be added in Phase 5 (US3)
