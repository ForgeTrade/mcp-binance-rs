//! Bearer Token Authentication Middleware
//!
//! Validates Authorization: Bearer <token> headers against configured tokens.
//! Tokens are loaded from HTTP_BEARER_TOKEN environment variable.

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Metadata associated with an authentication token
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetadata {
    /// Human-readable name/identifier for this token
    pub name: String,
    /// When this token was created (for auditing)
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Thread-safe store for valid authentication tokens
///
/// Tokens are stored as SHA-256 hashes for security.
/// Use `add_token()` to register new tokens, `validate()` to check requests.
#[derive(Debug, Clone)]
pub struct TokenStore {
    /// Map of token hash → metadata
    tokens: Arc<RwLock<HashMap<String, TokenMetadata>>>,
}

impl TokenStore {
    /// Create a new empty token store
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a token to the store
    ///
    /// ## Arguments
    ///
    /// - `token`: The raw token string (will be hashed)
    /// - `name`: Human-readable identifier for this token
    pub fn add_token(&self, token: &str, name: String) {
        let hash = Self::hash_token(token);
        let metadata = TokenMetadata {
            name,
            created_at: chrono::Utc::now(),
        };

        let mut tokens = self.tokens.write().expect("Token store lock poisoned");
        tokens.insert(hash, metadata);
    }

    /// Validate a token from an HTTP request
    ///
    /// Returns `Ok(metadata)` if token is valid, `Err(StatusCode)` otherwise
    pub fn validate(&self, token: &str) -> Result<TokenMetadata, StatusCode> {
        let hash = Self::hash_token(token);
        let tokens = self.tokens.read().expect("Token store lock poisoned");

        tokens.get(&hash).cloned().ok_or(StatusCode::UNAUTHORIZED)
    }

    /// Hash a token using SHA-256
    fn hash_token(token: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract and validate Bearer token from Authorization header
///
/// ## Header Format
///
/// ```text
/// Authorization: Bearer <token>
/// ```
///
/// ## Returns
///
/// - `Ok(())` if token is valid
/// - `Err(Response)` with appropriate HTTP status if invalid/missing
#[allow(clippy::result_large_err)]
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, Response> {
    let auth_header = headers
        .get("authorization")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response())?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid Authorization header encoding",
            )
                .into_response()
        })?;

    // Check for "Bearer " prefix (case-insensitive)
    if !auth_header.to_lowercase().starts_with("bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authorization header must use Bearer scheme",
        )
            .into_response());
    }

    // Extract token (skip "Bearer " prefix)
    let token = auth_header[7..].trim().to_string();

    if token.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Empty bearer token").into_response());
    }

    Ok(token)
}

/// Axum middleware function to validate bearer tokens
///
/// ## Usage
///
/// ```rust,no_run
/// use axum::{Router, middleware};
/// use mcp_binance_server::http::middleware::auth::TokenStore;
///
/// let token_store = TokenStore::new();
/// token_store.add_token("secret123", "client1".to_string());
///
/// let app = Router::new()
///     .route("/api/endpoint", axum::routing::get(handler))
///     .layer(middleware::from_fn_with_state(
///         token_store,
///         validate_bearer_token
///     ));
/// ```
pub async fn validate_bearer_token(
    axum::extract::State(token_store): axum::extract::State<TokenStore>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let token = extract_bearer_token(request.headers())?;

    // Validate token against store
    token_store
        .validate(&token)
        .map_err(|status| (status, "Invalid or expired token").into_response())?;

    // Token is valid, proceed with request
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_store() {
        let store = TokenStore::new();
        store.add_token("test_token_123", "Test Client".to_string());

        // Valid token
        assert!(store.validate("test_token_123").is_ok());

        // Invalid token
        assert_eq!(store.validate("wrong_token"), Err(StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn test_token_hashing() {
        let hash1 = TokenStore::hash_token("same_token");
        let hash2 = TokenStore::hash_token("same_token");
        let hash3 = TokenStore::hash_token("different_token");

        // Same token produces same hash
        assert_eq!(hash1, hash2);

        // Different tokens produce different hashes
        assert_ne!(hash1, hash3);
    }
}
