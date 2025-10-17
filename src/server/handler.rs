//! MCP ServerHandler trait implementation
//!
//! Implements the MCP protocol ServerHandler trait for the Binance server.
//! Provides server info, capabilities, and lifecycle management.

use crate::server::BinanceServer;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ToolsCapability,
};
use rmcp::tool_handler;

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BinanceServer {
    /// Returns server information and capabilities
    ///
    /// This is called during MCP initialization to communicate server metadata
    /// and supported features to the client.
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "mcp-binance-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("Binance MCP Server".to_string()),
                website_url: Some("https://github.com/tradeforge/mcp-binance-rs".to_string()),
                icons: None,
            },
            instructions: Some(
                "Binance MCP Server for trading and market data. \
                Provides tools for accessing Binance API endpoints including \
                market data, account information, and trading operations."
                    .to_string(),
            ),
        }
    }
}
