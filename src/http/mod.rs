//! HTTP REST API and WebSocket server module
//!
//! Provides HTTP transport for MCP server alongside stdio transport.
//! Includes REST endpoints for market data, orders, and account info,
//! plus WebSocket streaming for real-time data.
//!
//! ## Architecture
//!
//! - `routes/`: HTTP REST endpoint handlers
//! - `middleware/`: Authentication, rate limiting, CORS
//! - `websocket/`: WebSocket client connections to Binance

#[cfg(feature = "http-api")]
pub mod middleware;
#[cfg(feature = "http-api")]
pub mod routes;
#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "http-api")]
pub use routes::create_router;
