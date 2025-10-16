//! CORS (Cross-Origin Resource Sharing) Middleware
//!
//! Configures CORS headers to allow browser-based clients to access the API.

use tower_http::cors::{Any, CorsLayer};

/// Create CORS middleware layer with permissive settings for development
///
/// ## Configuration
///
/// - Allows all origins (`Access-Control-Allow-Origin: *`)
/// - Allows all methods (GET, POST, DELETE, OPTIONS, etc.)
/// - Allows all headers
/// - Exposes all headers to clients
/// - Max age: 3600 seconds (1 hour) for preflight cache
///
/// ## Production Note
///
/// For production, consider restricting:
/// - `allow_origin()` to specific domains
/// - `allow_headers()` to required headers only
/// - `allow_methods()` to needed methods only
///
/// ## Example
///
/// ```rust,no_run
/// use axum::Router;
/// use mcp_binance_server::http::middleware::create_cors_layer;
///
/// let app = Router::new()
///     .route("/api/endpoint", axum::routing::get(handler))
///     .layer(create_cors_layer());
/// ```
pub fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        // Allow all origins (permissive for development)
        .allow_origin(Any)
        // Allow all HTTP methods
        .allow_methods(Any)
        // Allow all headers
        .allow_headers(Any)
        // Expose all response headers to browser
        .expose_headers(Any)
        // Cache preflight for 1 hour
        .max_age(std::time::Duration::from_secs(3600))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_creation() {
        // Just verify it doesn't panic
        let _layer = create_cors_layer();
    }
}
