//! MCP Protocol Lifecycle Integration Tests
//!
//! Tests basic server initialization. Full MCP protocol flow is tested
//! via scripts/test_quickstart.sh which runs the actual server binary.

use mcp_binance_server::server::BinanceServer;
use rmcp::handler::server::ServerHandler;

#[test]
fn test_server_initialization() {
    // Create server instance
    let server = BinanceServer::new();

    // Get server info (not async)
    let info = server.get_info();

    // Verify protocol version
    assert_eq!(
        info.protocol_version,
        rmcp::model::ProtocolVersion::V_2024_11_05,
        "Protocol version should be 2024-11-05"
    );

    // Verify capabilities advertise tools
    assert!(
        info.capabilities.tools.is_some(),
        "Server should advertise tools capability"
    );

    // Verify server info
    assert_eq!(
        info.server_info.name, "mcp-binance-server",
        "Server name should be mcp-binance-server"
    );
    assert_eq!(
        info.server_info.version,
        env!("CARGO_PKG_VERSION"),
        "Server version should match package version"
    );
}

#[test]
fn test_initialization_without_credentials() {
    // Unset credentials (they may be in environment)
    unsafe {
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_SECRET_KEY");
    }

    // Create server - should succeed even without credentials
    let server = BinanceServer::new();

    // Server should initialize successfully
    let info = server.get_info();
    assert_eq!(info.server_info.name, "mcp-binance-server");
}

#[test]
fn test_server_clone_and_concurrent_access() {
    let server = BinanceServer::new();

    // Clone server (tests Arc sharing)
    let server_clone = server.clone();

    // Both should work independently
    let info1 = server.get_info();
    let info2 = server_clone.get_info();

    assert_eq!(info1.server_info.name, info2.server_info.name);
}
