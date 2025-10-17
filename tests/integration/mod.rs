//! Integration tests for MCP Binance Server
//!
//! This module contains comprehensive integration tests organized by feature area:
//! - REST API tests (market data, orders, account endpoints)
//! - WebSocket tests (ticker, depth, user data streams)
//! - Security tests (authentication, credential protection)
//! - Error handling tests (network failures, API errors)
//! - Performance tests (response times, throughput, memory)
//!
//! All tests use Binance Testnet API exclusively to avoid production risks.

// Import common test utilities
#[path = "../common/mod.rs"]
mod common;

// Existing tests
mod mcp_lifecycle;
mod security;
mod server_time;

// New test modules (Phases 3-7)
mod error_handling;
pub mod performance;
pub mod rest_api;
mod security_extended;
pub mod websocket;
