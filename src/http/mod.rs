//! HTTP REST API and WebSocket server module
//!
//! Provides HTTP transport for MCP server alongside stdio transport.
//! Includes REST endpoints for market data, orders, and account info,
//! plus WebSocket streaming for real-time data.
//!
//! ## Architecture
//!
//! - `routes/`: HTTP REST endpoint handlers
//! - `middleware/`: Authentication, rate limiting, CORS
//! - `websocket/`: WebSocket client connections to Binance

use std::sync::Arc;

#[cfg(feature = "http-api")]
pub mod middleware;
#[cfg(feature = "http-api")]
pub mod routes;
#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "http-api")]
use crate::binance::client::BinanceClient;
#[cfg(feature = "http-api")]
use axum::Router;
#[cfg(feature = "http-api")]
pub use middleware::{
    RateLimiter, TokenStore, check_rate_limit, create_cors_layer, validate_bearer_token,
};

/// Maximum concurrent WebSocket connections (per SC-003 requirement)
#[cfg(all(feature = "http-api", feature = "websocket"))]
const MAX_WS_CONNECTIONS: usize = 50;

/// Shared application state passed to all HTTP handlers
///
/// ## Fields
///
/// - `binance_client`: Arc-wrapped Binance API client for making requests
/// - `token_store`: Arc-wrapped authentication token store
/// - `rate_limiter`: Arc-wrapped rate limiter for global request limiting
/// - `ws_connections`: Semaphore for limiting concurrent WebSocket connections (max 50)
///
/// ## Usage
///
/// Handlers can access state using `axum::extract::State`:
///
/// ```rust,no_run
/// use axum::extract::State;
/// use mcp_binance_server::http::AppState;
///
/// async fn handler(State(state): State<AppState>) {
///     let server_time = state.binance_client.get_server_time().await;
/// }
/// ```
#[cfg(feature = "http-api")]
#[derive(Clone)]
pub struct AppState {
    /// Binance API client for making requests
    pub binance_client: Arc<BinanceClient>,

    /// Authentication token store
    pub token_store: TokenStore,

    /// Rate limiter for global request limiting
    pub rate_limiter: RateLimiter,

    /// WebSocket connection limit semaphore (max 50 concurrent)
    #[cfg(feature = "websocket")]
    pub ws_connections: Arc<tokio::sync::Semaphore>,
}

/// Create the main HTTP router with all middleware and routes
///
/// ## Arguments
///
/// - `token_store`: Authentication token store
/// - `rate_limiter`: Rate limiter instance
///
/// ## Returns
///
/// Configured axum Router with:
/// - CORS middleware
/// - Authentication middleware
/// - Rate limiting middleware
/// - All REST API routes
/// - Health check endpoint
///
/// ## Example
///
/// ```rust,no_run
/// use mcp_binance_server::http::{create_router, TokenStore, RateLimiter};
///
/// # async fn example() {
/// let token_store = TokenStore::new();
/// let rate_limiter = RateLimiter::new(100);
///
/// let app = create_router(token_store, rate_limiter);
///
/// let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
///     .await
///     .unwrap();
/// axum::serve(listener, app).await.unwrap();
/// # }
/// ```
#[cfg(feature = "http-api")]
pub fn create_router(token_store: TokenStore, rate_limiter: RateLimiter) -> Router {
    use axum::middleware;

    // Create shared application state
    let state = AppState {
        binance_client: Arc::new(BinanceClient::new()),
        token_store: token_store.clone(),
        rate_limiter: rate_limiter.clone(),
        #[cfg(feature = "websocket")]
        ws_connections: Arc::new(tokio::sync::Semaphore::new(MAX_WS_CONNECTIONS)),
    };

    // Create API v1 routes (protected by auth)
    let api_routes = Router::new()
        // Market data endpoints (Phase 3 - US1)
        .route(
            "/ticker/price",
            axum::routing::get(routes::market_data::get_ticker_price),
        )
        .route(
            "/ticker/24hr",
            axum::routing::get(routes::market_data::get_ticker_24hr),
        )
        .route(
            "/klines",
            axum::routing::get(routes::market_data::get_klines),
        )
        .route("/depth", axum::routing::get(routes::market_data::get_depth))
        .route(
            "/trades",
            axum::routing::get(routes::market_data::get_trades),
        )
        // Order endpoints (Phase 4 - US2)
        .route(
            "/order",
            axum::routing::post(routes::orders::create_order)
                .delete(routes::orders::cancel_order)
                .get(routes::orders::query_order),
        )
        .route(
            "/openOrders",
            axum::routing::get(routes::orders::get_open_orders),
        )
        .route(
            "/allOrders",
            axum::routing::get(routes::orders::get_all_orders),
        )
        // Account endpoints (Phase 5 - US3)
        .route("/account", axum::routing::get(routes::account::get_account))
        .route(
            "/myTrades",
            axum::routing::get(routes::account::get_my_trades),
        )
        // User data stream endpoints (Phase 8 - US6)
        .route(
            "/userDataStream",
            axum::routing::post(routes::account::create_user_data_stream)
                .put(routes::account::keepalive_user_data_stream)
                .delete(routes::account::close_user_data_stream),
        )
        .with_state(state.clone());

    // Build main router with health check and API routes
    #[allow(unused_mut)]
    let mut router = Router::new()
        // Health check (no auth required)
        .route("/health", axum::routing::get(|| async { "OK" }))
        // Mount API routes under /api/v1
        .nest("/api/v1", api_routes);

    // Add WebSocket routes if websocket feature is enabled
    #[cfg(all(feature = "http-api", feature = "websocket"))]
    {
        router = router
            .route(
                "/ws/ticker/{symbol}",
                axum::routing::get(websocket::ticker_handler),
            )
            .route(
                "/ws/depth/{symbol}",
                axum::routing::get(websocket::depth_handler),
            )
            .route("/ws/user", axum::routing::get(websocket::user_data_handler));
    }

    // Add SSE routes for remote MCP access (T011 - Feature 009)
    // SSE endpoints are implemented in transport::sse::handlers module
    // Routes will be integrated in Phase 3 (T020-T022) when handlers are ready
    #[cfg(feature = "sse")]
    {
        // TODO: Merge SSE router when handlers module is implemented
        // Example: router = router.merge(crate::transport::sse::create_sse_router(state));
        tracing::debug!("SSE feature enabled - routes will be added in Phase 3");
    }

    router
        // Apply middleware layers (order matters: outer → inner)
        .layer(create_cors_layer()) // CORS (outermost)
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            check_rate_limit,
        )) // Rate limiting
        .layer(middleware::from_fn_with_state(
            token_store,
            validate_bearer_token,
        )) // Authentication (innermost for protected routes)
        .with_state(state)
}
