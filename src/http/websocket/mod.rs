//! WebSocket client for Binance real-time data streams
//!
//! Connects to Binance WebSocket API and broadcasts data to HTTP clients:
//! - Ticker streams: real-time price updates
//! - Depth streams: order book updates
//! - User data streams: order fills, balance updates
//!
//! ## Architecture
//!
//! - Single WebSocket connection per stream type
//! - tokio::sync::broadcast for fan-out to multiple subscribers
//! - Automatic reconnection with exponential backoff

#[cfg(all(feature = "http-api", feature = "websocket"))]
pub mod depth;
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub mod ticker;
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub mod user_data;

#[cfg(all(feature = "http-api", feature = "websocket"))]
pub use depth::depth_handler;
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub use ticker::ticker_handler;
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub use user_data::user_data_handler;
