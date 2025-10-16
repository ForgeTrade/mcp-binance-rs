//! MCP Binance Server Binary
//!
//! Entry point for the MCP Binance server. Supports two modes:
//! - stdio transport (default): Standard MCP stdio communication
//! - HTTP server (--http flag): REST API + WebSocket server

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

    // Check if --http flag is provided
    let args: Vec<String> = std::env::args().collect();
    let http_mode = args.iter().any(|arg| arg == "--http");

    if http_mode {
        #[cfg(feature = "http-api")]
        {
            run_http_server().await?;
        }
        #[cfg(not(feature = "http-api"))]
        {
            eprintln!("Error: HTTP mode requires 'http-api' feature to be enabled");
            eprintln!("Rebuild with: cargo build --features http-api");
            std::process::exit(1);
        }
    } else {
        run_stdio_server().await?;
    }

    Ok(())
}

/// Run MCP server with stdio transport (default mode)
async fn run_stdio_server() -> Result<(), Box<dyn std::error::Error>> {
    // Create BinanceServer instance and serve with stdio transport
    let service = BinanceServer::new().serve(stdio()).await?;

    tracing::info!("MCP server initialized with stdio transport, waiting for requests");

    // Wait for the service to complete (blocks until stdin closes)
    service.waiting().await?;

    // Graceful shutdown
    tracing::info!("MCP server shutting down gracefully");

    Ok(())
}

/// Run HTTP REST API server (requires --http flag and http-api feature)
#[cfg(feature = "http-api")]
async fn run_http_server() -> Result<(), Box<dyn std::error::Error>> {
    use mcp_binance_server::config::HttpConfig;
    use mcp_binance_server::http::{RateLimiter, TokenStore, create_router};

    // Load HTTP configuration from environment
    let config = HttpConfig::from_env()?;

    tracing::info!("Starting HTTP server on {}", config.addr);
    tracing::info!("Rate limit: {} req/min per client", config.rate_limit);
    tracing::info!(
        "Max WebSocket connections: {}",
        config.max_websocket_connections
    );

    // Initialize token store and load tokens from environment
    let token_store = TokenStore::new();
    token_store.add_token(&config.bearer_token, "env_token".to_string());
    tracing::info!("Loaded 1 bearer token from environment");

    // Create rate limiter
    let rate_limiter = RateLimiter::new(config.rate_limit);

    // Create HTTP router with middleware
    let app = create_router(token_store, rate_limiter);

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    tracing::info!("HTTP server listening on {}", config.addr);

    axum::serve(listener, app).await?;

    Ok(())
}
