# Research: Mainnet Support with Secure API Key Authentication

**Date**: 2025-10-19
**Branch**: `011-mainnet-api-auth`

## Overview

This document consolidates research findings for implementing per-session API credential management. Primary focus areas: session-scoped storage patterns, Rust async credential passing, structured error design, and format validation approaches.

## Research Questions

### R1: Session-Scoped Credential Storage Pattern

**Question**: What's the best Rust pattern for storing per-session credentials in an async context with automatic cleanup?

**Decision**: Extend existing `SessionManager` from Feature 010 with `HashMap<String, Credentials>` using `Arc<RwLock<>>` for thread-safe access.

**Rationale**:
- Feature 010 already implements SessionManager with session lifecycle management
- RwLock allows concurrent reads (credential status checks) with exclusive writes (configure/revoke)
- Automatic cleanup via existing session expiry mechanism (credentials cleared when session metadata cleared)
- Mcp-Session-Id (UUID v4) provides natural HashMap key for O(1) lookup
-

**Implementation Pattern**:
```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionMetadata>>>,
    credentials: Arc<RwLock<HashMap<String, Credentials>>>, // NEW: session_id -> credentials
    // ... existing fields
}

impl SessionManager {
    pub async fn store_credentials(&self, session_id: &str, creds: Credentials) -> Result<()> {
        let mut credentials = self.credentials.write().await;
        credentials.insert(session_id.to_string(), creds);
        Ok(())
    }

    pub async fn get_credentials(&self, session_id: &str) -> Option<Credentials> {
        let credentials = self.credentials.read().await;
        credentials.get(session_id).cloned()
    }

    pub async fn revoke_credentials(&self, session_id: &str) -> bool {
        let mut credentials = self.credentials.write().await;
        credentials.remove(session_id).is_some()
    }

    // Extend existing cleanup_expired_sessions to also clear credentials
    async fn cleanup_credentials(&self, session_id: &str) {
        let mut credentials = self.credentials.write().await;
        credentials.remove(session_id);
    }
}
```

**Alternatives Considered**:
1. **Thread-local storage** - Rejected: Tokio async tasks can migrate between threads
2. **Database persistence** - Rejected: Violates FR-004 (no disk persistence) and SC-004 (100ms performance)
3. **Mutex instead of RwLock** - Rejected: Credential status checks would block configuration operations

**Best Practices**:
- Clone `Credentials` on retrieval (avoid holding read lock during HTTP requests)
- Clear credentials in cleanup task (same mechanism that clears session metadata)
- Use `.cloned()` method to minimize lock hold time

---

### R2: BinanceClient Per-Session Credential Support

**Question**: How should BinanceClient accept per-session credentials without breaking existing global environment variable support?

**Decision**: Add optional `Credentials` parameter to BinanceClient methods requiring authentication, fallback to env vars if None.

**Rationale**:
- Backward compatibility: stdio mode uses env vars (existing behavior)
- SSE mode passes session credentials explicitly
- Type-safe via Option<&Credentials> parameter
- No breaking changes to public API

**Implementation Pattern**:
```rust
impl BinanceClient {
    // Public API calls (no auth) - unchanged
    pub async fn get_ticker_price(&self, symbol: &str) -> Result<TickerPrice> { /* ... */ }

    // Authenticated calls - add optional credentials parameter
    pub async fn get_account_info(&self, credentials: Option<&Credentials>) -> Result<AccountInfo> {
        let (api_key, api_secret, base_url) = match credentials {
            Some(creds) => (
                &creds.api_key,
                &creds.api_secret,
                creds.environment.base_url(),
            ),
            None => {
                // Fallback to environment variables (stdio mode)
                let key = std::env::var("BINANCE_API_KEY")?;
                let secret = std::env::var("BINANCE_API_SECRET")?;
                let base_url = "https://api.binance.com"; // Default mainnet
                (&key, &secret, base_url)
            }
        };

        // Existing HMAC signing logic uses extracted credentials
        self.signed_request("GET", "/api/v3/account", api_key, api_secret, base_url).await
    }
}
```

**Alternatives Considered**:
1. **Separate BinanceClient instances per session** - Rejected: High memory overhead (50 sessions = 50 HTTP clients)
2. **Request-scoped credential injection** - Rejected: Requires changing every tool call site
3. **Builder pattern for credentials** - Rejected: Overly complex for simple optional parameter

**Best Practices**:
- Use `Option<&Credentials>` (borrow, not owned) to avoid cloning on every API call
- Environment enum provides `base_url()` method: `Testnet -> "https://testnet.binance.vision"`, `Mainnet -> "https://api.binance.com"`
- Preserve existing error handling for missing env vars

---

### R3: Structured Error Design for Credential Errors

**Question**: What's the best structure for machine-readable error responses in MCP tools?

**Decision**: Define `CredentialError` enum with `to_json()` method producing `{"error_code": "...", "message": "..."}`

**Rationale**:
- MCP tools return `Result<T, McpError>` where McpError wraps JSON value
- Structured errors enable programmatic handling (ChatGPT can retry on rate limits, prompt for credentials, etc.)
- Error codes defined in FR-013 specification (6 codes)
- Deterministic error responses (same error code for same condition - NFR-005)

**Implementation Pattern**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialError {
    NotConfigured,
    InvalidApiKeyFormat(String),     // reason: "API key must be 64 alphanumeric characters"
    InvalidApiSecretFormat(String),  // reason
    InvalidEnvironment(String),      // reason: "Must be 'testnet' or 'mainnet'"
    BinanceApiError { message: String, code: i32 },
    RateLimitExceeded { retry_after: u64 },
}

impl CredentialError {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::NotConfigured => json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            }),
            Self::InvalidApiKeyFormat(reason) => json!({
                "error_code": "INVALID_API_KEY_FORMAT",
                "message": reason
            }),
            Self::InvalidApiSecretFormat(reason) => json!({
                "error_code": "INVALID_API_SECRET_FORMAT",
                "message": reason
            }),
            Self::InvalidEnvironment(reason) => json!({
                "error_code": "INVALID_ENVIRONMENT",
                "message": reason
            }),
            Self::BinanceApiError { message, code } => json!({
                "error_code": "BINANCE_API_ERROR",
                "message": message,
                "binance_code": code
            }),
            Self::RateLimitExceeded { retry_after } => json!({
                "error_code": "BINANCE_RATE_LIMIT",
                "message": "Rate limit exceeded",
                "retry_after": retry_after
            }),
        }
    }
}
```

**Alternatives Considered**:
1. **Plain string errors** - Rejected: Not machine-readable (FR-009 specifies structured format)
2. **HTTP-style numeric codes** - Rejected: MCP tools don't use HTTP semantics
3. **Nested error hierarchy** - Rejected: Adds complexity without benefit

**Best Practices**:
- Error codes are SCREAMING_SNAKE_CASE for grep-ability in logs
- Human-readable messages included for debugging
- Binance API errors preserve original error code (`binance_code` field)

---

### R4: Format Validation Strategy

**Question**: How to validate API key/secret format (64 alphanumeric chars) efficiently?

**Decision**: Use regex `^[A-Za-z0-9]{64}$` compiled once as lazy_static

**Rationale**:
- Simple, deterministic validation rule (FR-010)
- Regex compilation is one-time cost (lazy_static or once_cell)
- Synchronous validation <10ms (SC-007) - regex match is ~1μs
- No external crates needed beyond `regex 1.11+` (already in dependencies via other features)

**Implementation Pattern**:
```rust
use once_cell::sync::Lazy;
use regex::Regex;

static API_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z0-9]{64}$").unwrap()
});

static API_SECRET_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z0-9]{64}$").unwrap()
});

pub fn validate_api_key(key: &str) -> Result<(), CredentialError> {
    if !API_KEY_REGEX.is_match(key) {
        return Err(CredentialError::InvalidApiKeyFormat(
            "API key must be exactly 64 alphanumeric characters".to_string()
        ));
    }
    Ok(())
}

pub fn validate_api_secret(secret: &str) -> Result<(), CredentialError> {
    if !API_SECRET_REGEX.is_match(secret) {
        return Err(CredentialError::InvalidApiSecretFormat(
            "API secret must be exactly 64 alphanumeric characters".to_string()
        ));
    }
    Ok(())
}
```

**Alternatives Considered**:
1. **Manual char iteration** - Rejected: Less readable, similar performance
2. **Length check only** - Rejected: Doesn't validate alphanumeric constraint
3. **Async validation against Binance API** - Rejected: Adds latency (violates SC-004), out of scope per spec

**Best Practices**:
- Validate before storing credentials (fail-fast)
- Same regex for both key and secret (Binance uses same format)
- Lazy static avoids regex recompilation overhead

---

### R5: Environment Enum Design

**Question**: How should the Environment enum provide testnet/mainnet URLs?

**Decision**: Simple enum with `base_url()` method

**Rationale**:
- Type-safe environment selection (FR-002)
- URL mapping is pure function (no state)
- Serializable for JSON responses (get_credentials_status returns environment string)

**Implementation Pattern**:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Testnet,
    Mainnet,
}

impl Environment {
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::Testnet => "https://testnet.binance.vision",
            Self::Mainnet => "https://api.binance.com",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, CredentialError> {
        match s.to_lowercase().as_str() {
            "testnet" => Ok(Self::Testnet),
            "mainnet" => Ok(Self::Mainnet),
            _ => Err(CredentialError::InvalidEnvironment(
                "Environment must be 'testnet' or 'mainnet'".to_string()
            ))
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Testnet => write!(f, "testnet"),
            Self::Mainnet => write!(f, "mainnet"),
        }
    }
}
```

**Alternatives Considered**:
1. **String constants** - Rejected: Not type-safe
2. **Struct with URL field** - Rejected: Overengineered for two variants
3. **Feature flags** - Rejected: Environment is runtime configurable, not compile-time

**Best Practices**:
- serde `rename_all = "lowercase"` matches MCP tool parameter format
- `from_str()` provides case-insensitive parsing
- `Copy` trait avoids unnecessary cloning

---

## Summary of Decisions

| Research Area | Decision | Key Rationale |
|---------------|----------|---------------|
| Session Storage | HashMap<String, Credentials> in SessionManager with Arc<RwLock> | Extends Feature 010 infrastructure, thread-safe, automatic cleanup |
| BinanceClient API | Optional credentials parameter with env var fallback | Backward compatibility, type-safe, no breaking changes |
| Error Structure | CredentialError enum with to_json() method | Machine-readable, 6 error codes per FR-013, MCP-compliant |
| Format Validation | Regex `^[A-Za-z0-9]{64}$` via lazy_static | Synchronous <10ms, deterministic, simple |
| Environment Design | Enum with base_url() method | Type-safe, serializable, pure function |

**No unresolved technical unknowns remain.** All decisions align with constitution principles:
- Security-first: No credential logging, session isolation
- Type safety: Enum for Environment, structured errors
- Async-first: RwLock for concurrent access
- Modular: Extends existing SessionManager without new feature gates

**Ready to proceed to Phase 1: Design & Contracts**
