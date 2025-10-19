//! HTTP middleware for authentication, rate limiting, and CORS
//!
//! Provides tower middleware layers:
//! - Bearer token authentication
//! - Rate limiting (100 req/min per client)
//! - CORS headers for browser clients
//! - Request tracing

#[cfg(feature = "http-api")]
pub mod auth;
#[cfg(feature = "http-api")]
pub mod cors;
#[cfg(feature = "http-api")]
pub mod rate_limit;

#[cfg(feature = "http-api")]
pub use auth::{validate_bearer_token, TokenStore};
#[cfg(feature = "http-api")]
pub use cors::create_cors_layer;
#[cfg(feature = "http-api")]
pub use rate_limit::{check_rate_limit, RateLimiter};
