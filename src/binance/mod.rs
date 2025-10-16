//! Binance API Client
//!
//! This module contains the HTTP client for Binance API integration.

pub mod client;
pub mod types;

// Re-export commonly used types
pub use client::BinanceClient;
pub use types::ServerTimeResponse;
