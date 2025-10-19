//! MCP Binance Server Binary
//!
//! Entry point for the MCP Binance server. Supports three modes:
//! - stdio transport (default): Standard MCP stdio communication
//! - HTTP server (--http flag): REST API + WebSocket server
//! - SSE transport (--mode sse): Server-Sent Events for cloud deployment

use mcp_binance_server::server::BinanceServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Standard main entry point (stdio or standalone SSE server)
///
/// Disabled when shuttle-runtime feature is enabled (Shuttle provides its own main)
#[cfg(not(feature = "shuttle-runtime"))]
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

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let http_mode = args.iter().any(|arg| arg == "--http");

    // Parse --mode flag (T012)
    let mode = args.iter()
        .position(|arg| arg == "--mode")
        .and_then(|pos| args.get(pos + 1))
        .map(|s| s.as_str());

    // Route to appropriate transport mode
    match (http_mode, mode) {
        (true, _) => {
            // Legacy --http flag support
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
        }
        (false, Some("sse")) => {
            #[cfg(feature = "sse")]
            {
                run_sse_server().await?;
            }
            #[cfg(not(feature = "sse"))]
            {
                eprintln!("Error: SSE mode requires 'sse' feature to be enabled");
                eprintln!("Rebuild with: cargo build --features sse");
                std::process::exit(1);
            }
        }
        (false, Some("stdio")) | (false, None) => {
            // Default: stdio transport
            run_stdio_server().await?;
        }
        (false, Some(unknown)) => {
            eprintln!("Error: Unknown mode '{}'", unknown);
            eprintln!("Valid modes: stdio (default), sse");
            eprintln!("Usage: {} [--mode <MODE>]", args[0]);
            std::process::exit(1);
        }
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

/// Run MCP server with SSE transport (requires --mode sse and sse feature)
///
/// SSE transport enables remote HTTPS access to the MCP server, suitable for
/// cloud deployment on platforms like Shuttle.dev.
///
/// ## Implementation Status
///
/// - [x] T012: CLI flag parsing and mode routing
/// - [x] T020-T023: SSE endpoint handlers (Phase 3) - MVP complete
/// - [x] T032: Shuttle runtime integration (Phase 4)
#[cfg(feature = "sse")]
async fn run_sse_server() -> Result<(), Box<dyn std::error::Error>> {
    // Parse port from command line (default 8000)
    let args: Vec<String> = std::env::args().collect();
    let port = args
        .iter()
        .position(|arg| arg == "--port")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);

    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Starting SSE server on {}", addr);

    // Create router
    let app = create_sse_router();

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Streamable HTTP server ready - listening on {}", addr);
    tracing::info!("MCP endpoint: POST http://{}/mcp (use 'initialize' method to create session)", addr);
    tracing::info!("Health check: http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates SSE router with all endpoints
///
/// Used by both standalone server (`run_sse_server`) and Shuttle runtime.
#[cfg(feature = "sse")]
fn create_sse_router() -> axum::Router {
    use mcp_binance_server::server::BinanceServer;
    use mcp_binance_server::transport::sse::{
        SessionManager, SseState, message_post, tools_list, server_info,
    };

    // Create session manager and MCP server
    let session_manager = SessionManager::new();
    let mcp_server = BinanceServer::new();
    let state = SseState::new(session_manager, mcp_server);

    // Create router with Streamable HTTP endpoints (March 2025 spec)
    // Removed legacy SSE GET handshake endpoints (/sse, /mcp/sse)
    // Consolidated to single POST /mcp endpoint with Mcp-Session-Id header
    axum::Router::new()
        .route("/", axum::routing::get(server_info))
        // Streamable HTTP transport (March 2025 spec) - POST only
        .route("/mcp", axum::routing::post(message_post))
        // Backward compatibility - alias to /mcp
        .route("/messages", axum::routing::post(message_post))
        // Additional endpoints
        .route("/tools/list", axum::routing::post(tools_list))
        .route("/health", axum::routing::get(|| async { "OK" }))
        .with_state(state)
}

/// Shuttle.dev runtime entry point (T032)
///
/// Enabled via `shuttle-runtime` feature flag for Shuttle deployment.
/// Uses SecretStore for Binance API credentials (T033).
///
/// ## Deployment
///
/// ```bash
/// shuttle deploy --features sse
/// ```
///
/// ## Secrets Configuration (T033)
///
/// ```bash
/// shuttle secrets add BINANCE_API_KEY=your_api_key
/// shuttle secrets add BINANCE_API_SECRET=your_secret_key
/// ```
#[cfg(all(feature = "sse", feature = "shuttle-runtime"))]
#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secret_store: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // T033: Load Binance API credentials from Shuttle secrets
    // These will be available to BinanceServer via environment variables
    // SAFETY: Setting environment variables before spawning threads is safe
    // Shuttle runtime is single-threaded at this point
    if let Some(api_key) = secret_store.get("BINANCE_API_KEY") {
        unsafe {
            std::env::set_var("BINANCE_API_KEY", api_key);
        }
        tracing::info!("Loaded BINANCE_API_KEY from Shuttle secrets");
    }
    if let Some(api_secret) = secret_store.get("BINANCE_API_SECRET") {
        unsafe {
            std::env::set_var("BINANCE_API_SECRET", api_secret);
        }
        tracing::info!("Loaded BINANCE_API_SECRET from Shuttle secrets");
    }

    tracing::info!("Starting MCP Binance Server on Shuttle.dev");

    // Create SSE router (reuses same router as standalone mode)
    let router = create_sse_router();

    // T035: Shuttle handles graceful shutdown automatically
    Ok(router.into())
}
