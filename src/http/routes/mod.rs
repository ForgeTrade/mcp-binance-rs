//! HTTP REST API route handlers
//!
//! Provides REST endpoints for Binance API operations:
//! - Market data: prices, orderbook, trades, candles
//! - Orders: create, cancel, query
//! - Account: balance, positions

#[cfg(feature = "http-api")]
pub mod market_data;
#[cfg(feature = "http-api")]
pub mod orders;

// Account routes will be added in Phase 5 (US3)
