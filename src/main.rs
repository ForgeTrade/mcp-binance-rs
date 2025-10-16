//! MCP Binance Server Binary
//!
//! Entry point for the MCP Binance server. This binary initializes the server
//! with stdio transport and handles graceful shutdown.

use mcp_binance_server::server::BinanceServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with env filter
    // Logs go to stderr (not stdout, which is used for MCP protocol)
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(false), // No ANSI colors for cleaner logs
        )
        .init();

    tracing::info!("Starting MCP Binance Server v{}", env!("CARGO_PKG_VERSION"));

    // Create BinanceServer instance and serve with stdio transport
    let service = BinanceServer::new().serve(stdio()).await?;

    tracing::info!("MCP server initialized, waiting for requests");

    // Wait for the service to complete (blocks until stdin closes)
    service.waiting().await?;

    // Graceful shutdown
    tracing::info!("MCP server shutting down gracefully");

    Ok(())
}
