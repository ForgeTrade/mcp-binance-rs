# Data Model: Mainnet Support with Secure API Key Authentication

**Date**: 2025-10-19
**Branch**: `011-mainnet-api-auth`

## Overview

This document defines the data structures for session-scoped credential management. All entities are stored in-memory with no disk persistence (FR-004).

## Core Entities

### Credentials

Session-scoped credential storage for Binance API authentication.

**Fields**:
- `api_key: String` - Binance API key (validated: 64 alphanumeric characters)
- `api_secret: String` - Binance API secret (validated: 64 alphanumeric characters)
- `environment: Environment` - Target Binance environment (Testnet | Mainnet)
- `configured_at: DateTime<Utc>` - ISO8601 timestamp of credential configuration
- `session_id: String` - UUID v4 session ID for isolation (references Mcp-Session-Id header)

**Validation Rules** (from FR-010):
- `api_key`: Must match regex `^[A-Za-z0-9]{64}$`
- `api_secret`: Must match regex `^[A-Za-z0-9]{64}$`
- `environment`: Must be `Testnet` or `Mainnet` (enum constraint)
- `session_id`: Must be valid UUID v4 format

**Lifecycle**:
1. Created via `configure_credentials` tool (synchronous format validation)
2. Stored in SessionManager's `credentials: HashMap<String, Credentials>`
3. Retrieved for authenticated API calls (BinanceClient.get_account_info, etc.)
4. Cleared when session ends or `revoke_credentials` called

**Security Constraints** (from NFR-001, NFR-002):
- Never persisted to disk
- `api_secret` never logged at any log level
- Cleared from memory immediately on session end

**Rust Type**:
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: String,
    pub environment: Environment,
    pub configured_at: DateTime<Utc>,
    #[serde(skip)]  // Never serialize session_id in responses
    pub session_id: String,
}

impl Credentials {
    pub fn new(
        api_key: String,
        api_secret: String,
        environment: Environment,
        session_id: String,
    ) -> Self {
        Self {
            api_key,
            api_secret,
            environment,
            configured_at: Utc::now(),
            session_id,
        }
    }

    /// Returns first 8 characters of API key for status display (NFR-003)
    pub fn key_prefix(&self) -> String {
        self.api_key.chars().take(8).collect()
    }
}
```

---

### Environment

**Module Location**: `src/types.rs` (shared domain types, not error types)

Trading environment enumeration for Binance API endpoint selection.

**Values**:
- `Testnet` - Binance testnet environment
- `Mainnet` - Binance production environment

**Behavior**:
- Provides `base_url()` method for endpoint resolution
- Serializes as lowercase string ("testnet" | "mainnet") for MCP responses
- Public tools (get_ticker, get_klines) always use Mainnet regardless of session credentials (FR-011)

**Endpoint Mapping** (from FR-002):
- `Testnet` → `https://testnet.binance.vision`
- `Mainnet` → `https://api.binance.com`

**Rust Type**:
```rust
use serde::{Deserialize, Serialize};
use std::fmt;

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

---

### CredentialError

Structured error response for credential-related failures.

**Variants** (from FR-013 error code catalog):
- `NotConfigured` - No credentials configured in session
- `InvalidApiKeyFormat(String)` - API key validation failed (not 64 alphanumeric)
- `InvalidApiSecretFormat(String)` - API secret validation failed (not 64 alphanumeric)
- `InvalidEnvironment(String)` - Environment not "testnet" or "mainnet"
- `BinanceApiError { message: String, code: i32 }` - Binance API rejected credentials
- `RateLimitExceeded { retry_after: u64 }` - Binance rate limit exceeded

**Error Code Mapping** (from FR-013):

| Variant | error_code | HTTP Analogy | User Action |
|---------|-----------|--------------|-------------|
| NotConfigured | CREDENTIALS_NOT_CONFIGURED | 401 Unauthorized | Call configure_credentials |
| InvalidApiKeyFormat | INVALID_API_KEY_FORMAT | 400 Bad Request | Fix API key format |
| InvalidApiSecretFormat | INVALID_API_SECRET_FORMAT | 400 Bad Request | Fix API secret format |
| InvalidEnvironment | INVALID_ENVIRONMENT | 400 Bad Request | Use "testnet" or "mainnet" |
| BinanceApiError | BINANCE_API_ERROR | 403 Forbidden | Check Binance API key permissions |
| RateLimitExceeded | BINANCE_RATE_LIMIT | 429 Too Many Requests | Wait retry_after seconds |

**JSON Response Format** (from FR-009):
```json
{
  "error_code": "<ERROR_CODE>",
  "message": "<human-readable description>",
  "binance_code": 123,      // Optional: only for BinanceApiError
  "retry_after": 60         // Optional: only for RateLimitExceeded
}
```

**Rust Type**:
```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialError {
    NotConfigured,
    InvalidApiKeyFormat { reason: String },
    InvalidApiSecretFormat { reason: String },
    InvalidEnvironment { reason: String },
    BinanceApiError { message: String, code: i32 },
    RateLimitExceeded { retry_after: u64 },
}

impl CredentialError {
    /// Converts error to structured JSON response (FR-009 compliance)
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::NotConfigured => json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            }),
            Self::InvalidApiKeyFormat { reason } => json!({
                "error_code": "INVALID_API_KEY_FORMAT",
                "message": reason
            }),
            Self::InvalidApiSecretFormat { reason } => json!({
                "error_code": "INVALID_API_SECRET_FORMAT",
                "message": reason
            }),
            Self::InvalidEnvironment { reason } => json!({
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

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_json())
    }
}
```

---

## Relationships

```
SessionManager (Feature 010)
  │
  ├── sessions: HashMap<String, SessionMetadata>   # Existing
  └── credentials: HashMap<String, Credentials>    # NEW: 1:1 with sessions
                      │
                      └── session_id (FK) → Mcp-Session-Id header

BinanceClient
  │
  └── authenticated methods receive Option<&Credentials>
        │
        └── uses api_key + api_secret + environment.base_url()
```

**Cardinality**:
- 1 Session : 0..1 Credentials (credentials optional, configured via tool)
- 1 Credentials : 1 Environment (required field, validated on configuration)
- SessionManager cleanup removes both session metadata AND credentials atomically

**Access Patterns**:
1. **Write**: `configure_credentials` → SessionManager.store_credentials()
2. **Read**: Account/trading tools → SessionManager.get_credentials() → BinanceClient(Option<&Credentials>)
3. **Delete**: `revoke_credentials` OR session cleanup → SessionManager.revoke_credentials()

---

## State Transitions

### Credentials Lifecycle

```
                    configure_credentials (valid format)
[No Credentials] ──────────────────────────────────────> [Credentials Configured]
                                                                    │
                                                                    │ revoke_credentials
                                                                    │ OR session ends
                                                                    ↓
                                                          [No Credentials]
```

**States**:
1. **No Credentials** (initial): Session exists, no credentials configured
   - `get_credentials_status` returns `{"configured": false}`
   - Account/trading tools return `CREDENTIALS_NOT_CONFIGURED` error

2. **Credentials Configured**: Valid credentials stored in SessionManager
   - `get_credentials_status` returns full status (environment, key_prefix, configured_at)
   - Account/trading tools execute with session credentials

**Transitions**:
- `configure_credentials(valid)` → No Credentials → Configured (synchronous format validation)
- `configure_credentials(invalid)` → No Credentials (error returned, state unchanged)
- `revoke_credentials` → Configured → No Credentials (immediate)
- Session cleanup → Configured → No Credentials (automatic when session ends)
- `configure_credentials` on Configured state → Overwrites existing (last write wins)

**Validation Gates** (from FR-010):
- Format validation (regex) at Configure Credentials → Configured transition
- API validation (Binance API) occurs asynchronously on first authenticated tool call (not during state transition)

---

## Storage Implementation

**In-Memory HashMap** (from research.md R1):
```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionMetadata>>>,   // Existing from Feature 010
    credentials: Arc<RwLock<HashMap<String, Credentials>>>,    // NEW for Feature 011
}
```

**Thread Safety**:
- `Arc` enables sharing across async tasks
- `RwLock` allows concurrent reads (status checks) with exclusive writes (configure/revoke)
- Clone `Credentials` on retrieval to avoid holding lock during HTTP requests

**Cleanup Strategy**:
1. Session expiry timer (existing from Feature 010)
2. When session expires: remove from both `sessions` AND `credentials` HashMaps atomically
3. Manual revoke: `revoke_credentials` removes from `credentials` HashMap only (session remains active)

**Memory Footprint**:
- Per session: ~200 bytes (2x 64-char strings + DateTime + UUID)
- Max 50 sessions (from Feature 010 SC-004): ~10KB total
- Negligible compared to HTTP client memory overhead

---

## Validation Summary

| Entity | Validation Rules | Error on Failure |
|--------|------------------|------------------|
| Credentials.api_key | Regex `^[A-Za-z0-9]{64}$` | INVALID_API_KEY_FORMAT |
| Credentials.api_secret | Regex `^[A-Za-z0-9]{64}$` | INVALID_API_SECRET_FORMAT |
| Credentials.environment | Enum (Testnet \| Mainnet) | INVALID_ENVIRONMENT |
| Credentials.session_id | UUID v4 format | (internal, not user-facing) |

**Validation Timing** (from clarifications):
- **Synchronous**: Format validation (regex) during `configure_credentials` (<10ms, SC-007)
- **Asynchronous**: API validation by Binance API on first authenticated tool call (out of scope for this feature)
