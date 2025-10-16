//! HTTP REST API route handlers
//!
//! Provides REST endpoints for Binance API operations:
//! - Market data: prices, orderbook, trades, candles
//! - Orders: create, cancel, query
//! - Account: balance, positions

use axum::Router;

/// Creates the main HTTP router with all REST API routes
///
/// ## Routes
///
/// - `GET /health` - Health check endpoint
/// - `GET /api/v1/market/*` - Market data endpoints
/// - `POST /api/v1/orders/*` - Order management endpoints
/// - `GET /api/v1/account/*` - Account information endpoints
pub fn create_router() -> Router {
    Router::new()
        // Health check (no auth required)
        .route("/health", axum::routing::get(|| async { "OK" }))
    // Market data routes will be added in Phase 3
    // Order routes will be added in Phase 4
    // Account routes will be added in Phase 5
}
