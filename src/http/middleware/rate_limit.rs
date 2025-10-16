//! Rate Limiting Middleware
//!
//! Limits requests per client to prevent abuse and ensure fair resource usage.

use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Error type for rate limit exceeded
#[derive(Debug, Clone, Copy)]
pub struct RateLimitExceeded;

/// Rate limiter using governor crate for per-server limits
///
/// ## Configuration
///
/// Rate limit is set via `HTTP_RATE_LIMIT` environment variable (default: 100 req/min).
/// Applied globally across all clients.
///
/// ## Future Enhancement
///
/// Consider per-IP or per-token rate limiting for finer control.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    /// Create a new rate limiter with specified requests per minute
    ///
    /// ## Arguments
    ///
    /// - `requests_per_minute`: Maximum requests allowed per minute
    ///
    /// ## Panics
    ///
    /// Panics if `requests_per_minute` is 0
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(requests_per_minute).expect("Rate limit must be greater than 0"),
        );

        Self {
            inner: Arc::new(GovernorRateLimiter::direct(quota)),
        }
    }

    /// Check if a request is allowed
    ///
    /// Returns `Ok(())` if allowed, `Err(RateLimitExceeded)` if rate limit exceeded
    pub fn check(&self) -> Result<(), RateLimitExceeded> {
        self.inner.check().map_err(|_| RateLimitExceeded)
    }
}

/// Create rate limiter middleware from configuration
///
/// ## Usage
///
/// ```rust,no_run
/// use axum::{Router, middleware};
/// use mcp_binance_server::http::middleware::rate_limit::RateLimiter;
///
/// let rate_limiter = RateLimiter::new(100); // 100 req/min
///
/// let app = Router::new()
///     .route("/api/endpoint", axum::routing::get(handler))
///     .layer(middleware::from_fn_with_state(
///         rate_limiter,
///         check_rate_limit
///     ));
/// ```
pub async fn check_rate_limit(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // Check rate limit
    if limiter.check().is_err() {
        return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    // Request allowed, proceed
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(100);

        // First request should succeed
        assert!(limiter.check().is_ok());
    }

    #[test]
    #[should_panic(expected = "Rate limit must be greater than 0")]
    fn test_zero_rate_limit_panics() {
        let _limiter = RateLimiter::new(0);
    }
}
