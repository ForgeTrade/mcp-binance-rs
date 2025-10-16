//! MCP Binance Server Library
//!
//! This library provides the core functionality for the Binance MCP server,
//! including MCP protocol handling, Binance API integration, and tool implementations.

pub mod binance;
pub mod config;
pub mod error;
#[cfg(feature = "http-api")]
pub mod http;
pub mod server;
pub mod tools;

// Re-export commonly used types
pub use error::McpError;
pub use server::BinanceServer;
