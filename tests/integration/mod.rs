//! Integration tests for MCP Binance Server
//!
//! These tests verify end-to-end behavior including protocol compliance,
//! tool execution, and security.
//!
//! Note: Full MCP protocol tests require RequestContext setup which is
//! complex. These tests focus on core functionality and security.

mod mcp_lifecycle;
mod security;
mod server_time;
