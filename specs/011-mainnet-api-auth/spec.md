# Feature Specification: Mainnet Support with Secure API Key Authentication

**Feature Branch**: `011-mainnet-api-auth`
**Created**: 2025-10-19
**Status**: Draft
**Input**: User description: "Feature: Mainnet Support with Secure API Key Authentication"

## Clarifications

### Session 2025-10-19

- Q: What format should error messages use for credential-related errors? → A: Structured JSON error objects with error codes for programmatic handling (e.g., `{"error_code": "CREDENTIALS_NOT_CONFIGURED", "message": "..."}`)
- Q: When should credential validation occur? → A: Validate format synchronously on configure_credentials, validate API credentials asynchronously on first API call

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure API Credentials for Session (Priority: P1)

Users need to configure their Binance API credentials to access account data and execute trades through ChatGPT MCP integration.

**Why this priority**: This is the core blocker preventing users from using account and trading tools (get_account_info, place_order, etc.). Without API key configuration, 7 of 18 tools return "API key not configured" errors.

**Independent Test**: Start new ChatGPT session connected to MCP server → Call `configure_credentials` tool with API key, secret, and environment (testnet/mainnet) → Call `get_account_info` tool → Verify account data returns successfully.

**Acceptance Scenarios**:

1. **Given** a new SSE session without credentials, **When** user calls `configure_credentials` with valid testnet API key and secret, **Then** subsequent `get_account_info` calls return testnet account data
2. **Given** a new SSE session without credentials, **When** user calls `configure_credentials` with valid mainnet API key and secret, **Then** subsequent `place_order` calls execute on mainnet
3. **Given** a session with testnet credentials configured, **When** user calls `configure_credentials` with mainnet credentials, **Then** environment switches to mainnet and all subsequent API calls use mainnet endpoints
4. **Given** a session with credentials configured, **When** user calls `get_credentials_status`, **Then** response shows environment (testnet/mainnet), key prefix (first 8 chars), and configuration timestamp

---

### User Story 2 - Secure Credential Isolation (Priority: P1)

Each ChatGPT session must have isolated credentials that are never persisted to disk or shared between sessions.

**Why this priority**: Security requirement - API keys control real money. Credential leakage between sessions or persistence to disk creates unacceptable security risk.

**Independent Test**: Configure credentials in Session A → Start new Session B → Verify Session B cannot access Session A's credentials → Restart server → Verify credentials are not persisted across server restarts.

**Acceptance Scenarios**:

1. **Given** Session A with configured credentials, **When** Session B is created, **Then** Session B must call `configure_credentials` independently
2. **Given** a session with configured credentials, **When** session ends (connection closed), **Then** credentials are immediately cleared from memory
3. **Given** configured credentials in any session, **When** server restarts, **Then** no credentials persist to disk or configuration files
4. **Given** two concurrent sessions with different credentials, **When** both call `get_account_info`, **Then** each receives their own account data without cross-contamination

---

### User Story 3 - Revoke Credentials from Session (Priority: P2)

Users need ability to revoke API credentials from current session without closing the connection.

**Why this priority**: Nice-to-have for security-conscious users who want to clear credentials mid-session. Lower priority than configuration itself.

**Independent Test**: Configure credentials in session → Verify `get_account_info` works → Call `revoke_credentials` → Verify `get_account_info` returns structured JSON error with error_code "CREDENTIALS_NOT_CONFIGURED".

**Acceptance Scenarios**:

1. **Given** a session with configured credentials, **When** user calls `revoke_credentials`, **Then** subsequent account/trading tool calls return structured JSON error with error_code "CREDENTIALS_NOT_CONFIGURED"
2. **Given** a session with revoked credentials, **When** user calls `configure_credentials` again, **Then** credentials are reconfigured and tools work again
3. **Given** a session with revoked credentials, **When** user calls `get_credentials_status`, **Then** response shows "not configured" status

---

### User Story 4 - Environment-Specific Tool Behavior (Priority: P2)

Users need clear indication of which Binance environment (testnet/mainnet) their tools are executing against to prevent accidental real trades.

**Why this priority**: Important for user confidence and mistake prevention, but configuration (P1) must exist first.

**Independent Test**: Configure testnet credentials → Call `place_order` with small test order → Verify order appears in Binance Testnet UI → Reconfigure with mainnet credentials → Call `get_account_info` → Verify account data shows mainnet balances.

**Acceptance Scenarios**:

1. **Given** testnet credentials configured, **When** user calls `place_order`, **Then** order executes on `https://testnet.binance.vision` and response indicates testnet environment
2. **Given** mainnet credentials configured, **When** user calls `get_account_info`, **Then** request goes to `https://api.binance.com` and response indicates mainnet environment
3. **Given** testnet credentials configured, **When** user calls `get_ticker` (public endpoint), **Then** ticker data comes from mainnet (public endpoints always use mainnet regardless of credential configuration)

---

### Edge Cases

- What happens when API key format is invalid (not 64 alphanumeric characters)?
  → `configure_credentials` returns structured JSON error immediately: `{"error_code": "INVALID_API_KEY_FORMAT", "message": "API key must be exactly 64 alphanumeric characters"}`

- What happens when API secret format is invalid (not 64 alphanumeric characters)?
  → `configure_credentials` returns structured JSON error immediately: `{"error_code": "INVALID_API_SECRET_FORMAT", "message": "API secret must be exactly 64 alphanumeric characters"}`

- How does system handle invalid API credentials (wrong key/secret)?
  → Format validation passes in `configure_credentials`, first account/trading tool call returns Binance API error: `{"error_code": "BINANCE_API_ERROR", "message": "Invalid API-key, IP, or permissions for action", "binance_code": -2015}`

- What happens when session expires mid-request?
  → Credentials are cleared immediately when session closes, in-flight requests may fail with connection error

- How does system handle concurrent credential configuration in same session?
  → Last write wins (credentials are session-scoped, no global state)

- What happens under memory pressure with many sessions?
  → SessionManager enforces max 50 concurrent sessions limit (SC-004 from Feature 010), credentials cleared with session cleanup

- How does system handle malformed environment value (not "testnet" or "mainnet")?
  → `configure_credentials` returns structured JSON error: `{"error_code": "INVALID_ENVIRONMENT", "message": "Environment must be 'testnet' or 'mainnet'"}`

- What happens when user revokes credentials during active request?
  → In-flight requests complete with existing credentials, subsequent requests fail with error_code "CREDENTIALS_NOT_CONFIGURED"

- How do public tools work without credentials configured?
  → Public tools (get_ticker, etc.) continue working without credential configuration

- What happens when rate limit is exceeded?
  → System returns Binance rate limit error: `{"error_code": "BINANCE_RATE_LIMIT", "message": "Rate limit exceeded", "binance_code": -1003, "retry_after": 60}`

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST store API credentials on a per-session basis using Mcp-Session-Id as isolation key
- **FR-002**: System MUST support both testnet (`https://testnet.binance.vision`) and mainnet (`https://api.binance.com`) environments via credential configuration
- **FR-003**: System MUST clear credentials from memory when SSE session ends or connection closes
- **FR-004**: System MUST NOT persist credentials to disk, environment variables, or configuration files at any time
- **FR-005**: Users MUST be able to configure credentials via `configure_credentials` MCP tool with parameters: api_key (string), api_secret (string), environment (enum: "testnet" | "mainnet")
- **FR-006**: Users MUST be able to view credential status via `get_credentials_status` MCP tool returning: configured (boolean), environment (string), key_prefix (string, first 8 chars), configured_at (ISO8601 timestamp)
- **FR-007**: Users MUST be able to revoke credentials via `revoke_credentials` MCP tool
- **FR-008**: System MUST route account/trading tool calls to correct endpoint based on configured environment
- **FR-009**: System MUST return structured JSON error objects for all credential-related errors with format: `{"error_code": "<ERROR_CODE>", "message": "<human-readable message>"}`
- **FR-010**: System MUST validate API key format (64 chars alphanumeric) and secret format (64 chars alphanumeric) synchronously during `configure_credentials` call, before storing credentials
- **FR-011**: Public data tools (get_ticker, get_klines, get_order_book, etc.) MUST continue using mainnet endpoints regardless of credential configuration
- **FR-012**: System MUST enforce rate limiting per Binance API rules: 1200 req/min for signed endpoints, 6000 req/min for public endpoints (global rate limiting across all sessions, as Binance API limits are IP-based not credential-based)
- **FR-013**: System MUST use standardized error codes for credential errors: "CREDENTIALS_NOT_CONFIGURED" (no credentials in session), "INVALID_API_KEY_FORMAT" (key not 64 alphanumeric), "INVALID_API_SECRET_FORMAT" (secret not 64 alphanumeric), "INVALID_ENVIRONMENT" (environment not testnet/mainnet), "BINANCE_API_ERROR" (Binance API rejected credentials), "BINANCE_RATE_LIMIT" (rate limit exceeded)

### Key Entities

- **Credentials**: Session-scoped credential storage
  - `api_key: String` - Binance API key (64 chars)
  - `api_secret: String` - Binance API secret (64 chars)
  - `environment: Environment` - Enum: Testnet | Mainnet
  - `configured_at: DateTime<Utc>` - Timestamp of configuration
  - `session_id: String` - UUID v4 session ID for isolation

- **Environment**: Trading environment enumeration
  - `Testnet` - Uses `https://testnet.binance.vision`
  - `Mainnet` - Uses `https://api.binance.com`

- **CredentialError**: Structured error response
  - `error_code: String` - Machine-readable error identifier (see FR-013 for catalog)
  - `message: String` - Human-readable error description
  - `binance_code: Option<i32>` - Binance API error code (if applicable)
  - `retry_after: Option<u64>` - Retry delay in seconds (for rate limit errors)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of account/trading tools (7 tools: get_account_info, get_account_trades, place_order, get_order, cancel_order, get_open_orders, get_all_orders) return valid data when credentials configured
- **SC-002**: 0% credential leakage between sessions across 100 concurrent session test
- **SC-003**: 0% credential persistence to disk verified via filesystem monitoring during 1000 credential configuration cycles
- **SC-004**: Credential configuration completes within 100ms (in-memory storage only, excluding API validation which occurs on first tool call)
- **SC-005**: Environment switch (testnet → mainnet) takes effect within 1 API call (no caching delay)
- **SC-006**: All tools return structured JSON error with error_code "CREDENTIALS_NOT_CONFIGURED" within 50ms when credentials not configured (fast-fail)
- **SC-007**: Format validation errors (invalid key/secret format) are detected synchronously during `configure_credentials` call with <10ms latency

## Non-Functional Requirements

- **NFR-001**: Credential storage uses secure memory practices (no logging of secrets, cleared on session end)
- **NFR-002**: API secrets are never logged at any log level
- **NFR-003**: Credential status endpoint shows only first 8 characters of API key (security by obscurity)
- **NFR-004**: Implementation adds zero latency overhead to public data tools (no credential check for unsigned endpoints)
- **NFR-005**: Error responses are deterministic and testable (same error_code for same failure condition)

## Dependencies

- Feature 010 (Streamable HTTP Transport) - Session management infrastructure (Mcp-Session-Id)
- Existing BinanceClient in `src/binance/client.rs` - Must be refactored to support per-session credentials instead of global environment variables
- SessionManager in `src/transport/sse/session.rs` - Extended to store Credentials struct

## Out of Scope

- **API key validation against Binance API during configure_credentials** - First tool call validates credentials naturally via API error (asynchronous validation)
- **Credential encryption at rest** - Credentials are never persisted, so encryption is N/A
- **Multi-user authentication** - Each session is isolated; no user login or account management
- **API key rotation** - Users manually reconfigure credentials when rotating keys
- **Credential recovery** - Sessions are ephemeral; lost credentials require reconfiguration
- **Permission scoping** - Binance API key permissions are configured in Binance UI, not by this server
