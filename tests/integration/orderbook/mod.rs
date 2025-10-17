//! Integration tests for orderbook depth analysis feature
//!
//! Tests WebSocket connectivity, rate limiting, and metrics calculations.

#[cfg(feature = "orderbook")]
pub mod websocket_reconnection;

#[cfg(feature = "orderbook")]
pub mod rate_limit;

#[cfg(feature = "orderbook")]
pub mod metrics;
