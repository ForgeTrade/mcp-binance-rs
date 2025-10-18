# Research: SSE Transport for Cloud Deployment

**Date**: 2025-10-18
**Feature**: 009-specify-scripts-bash
**Purpose**: Resolve NEEDS CLARIFICATION items from Technical Context

## Research Questions

### Q1: rmcp SSE API Stability (NEEDS CLARIFICATION)

**Question**: Is the rmcp 0.8.1 SSE server API stable and production-ready?

**Decision**: ✅ Use rmcp 0.8.1 with `transport-sse-server` feature

**Rationale**:
- rmcp 0.8.1 is the latest stable version (verified via context7)
- Shuttle's own mcp-sse-oauth example uses rmcp 0.5+ with `transport-sse-server` feature
- The rust-sdk repository shows active SSE server examples (`servers_counter_sse`, `servers_simple_auth_sse`, `servers_complex_auth_sse`)
- API includes `SseServer::new()` which returns `(SseServer, Router)` - the Router integrates directly with Axum
- No deprecation warnings or breaking changes noted in documentation

**Alternatives Considered**:
1. **Implement custom SSE protocol** - REJECTED: Reinventing the wheel, MCP protocol compliance would be manual
2. **Wait for rmcp 1.0** - REJECTED: 0.8.1 is production-ready per Shuttle examples
3. **Use streamable HTTP instead of SSE** - REJECTED: SSE is standard for MCP remote servers

**Implementation Notes**:
```rust
use rmcp::transport::sse_server::{SseServer, SseServerConfig};

let config = SseServerConfig {
    bind: addr,
    sse_path: "/mcp/sse".to_string(),
    post_path: "/mcp/message".to_string(),
    ct: CancellationToken::new(),
    sse_keep_alive: Some(Duration::from_secs(30)),
};

let (sse_server, router) = SseServer::new(config);
// router is axum::Router - can merge with existing HTTP routes
```

---

### Q2: SSE-Specific Middleware Requirements (NEEDS CLARIFICATION)

**Question**: What Axum/Tower middleware is required for SSE transport?

**Decision**: ✅ Minimal middleware - SSE server handles most concerns internally

**Rationale**:
- rmcp's `SseServer::new()` returns a pre-configured Axum router with built-in middleware
- Shuttle example shows only CORS middleware needed for browser clients:
  ```rust
  tower_http::cors::CorsLayer // Only for web browser access
  ```
- SSE keep-alive handled by `sse_keep_alive` config parameter (30s default)
- Authentication/authorization is separate concern (see Q3)

**Middleware Stack**:
1. **CORS** (optional): `tower_http::cors::CorsLayer` - only if browser clients need access
2. **Logging**: Existing `tracing` integration works automatically
3. **Rate Limiting**: Inherit existing GCRA limiter from Binance API layer (no SSE-specific limiting needed)

**Alternatives Considered**:
1. **Custom SSE keep-alive middleware** - REJECTED: Built into `SseServerConfig`
2. **Connection pooling middleware** - REJECTED: Not applicable to SSE (one connection per client)
3. **Request timeout middleware** - REJECTED: SSE connections are long-lived by design

**Implementation Notes**:
- No special middleware needed beyond existing HTTP server setup
- SSE endpoints `/mcp/sse` and `/mcp/message` are pre-configured by rmcp
- Can merge SSE router with existing HTTP API routes using `Router::merge()`

---

### Q3: OAuth2 Authentication Requirements (NEEDS CLARIFICATION)

**Question**: Is OAuth2 required for production deployment or optional for MVP?

**Decision**: ⚠️ OAuth2 is OPTIONAL for MVP, RECOMMENDED for production

**Rationale**:
- **MVP Path (No OAuth)**: Shuttle deployment can use IP whitelisting + API key environment variables for security
- **Production Path (OAuth2)**: Shuttle's mcp-sse-oauth example demonstrates full OAuth2 PKCE implementation
- **Security Posture**:
  - Without OAuth: Rely on HTTPS + Shuttle secrets + network-level access control
  - With OAuth: Client-specific token validation + revocation capability
- **Complexity Trade-off**: OAuth2 adds ~500-800 LOC + OAuth endpoints + token storage

**MVP Implementation (No OAuth)**:
- Deploy SSE server with HTTPS-only access (Shuttle automatic SSL)
- Store Binance API keys in Shuttle secrets store
- Access control via Shuttle's built-in authentication (if available) or IP whitelist
- Claude Desktop connects directly to SSE endpoint URL

**Production Implementation (OAuth2 PKCE)**:
```rust
// Add to Cargo.toml
rmcp = { version = "0.8.1", features = ["transport-sse-server", "auth"] }

// OAuth endpoints from Shuttle example:
// POST /oauth/register - Client registration
// GET /oauth/authorize - Authorization code flow
// POST /oauth/token - Token exchange
// Middleware validates Bearer tokens on /mcp/sse and /mcp/message
```

**Decision for Initial Implementation**:
- **Phase 1 (MVP)**: No OAuth - simple HTTPS deployment
- **Phase 2 (Production Hardening)**: Add OAuth2 PKCE as optional feature flag

**Alternatives Considered**:
1. **API Key Header Authentication** - REJECTED: Not MCP standard, client support unclear
2. **mTLS (Mutual TLS)** - REJECTED: Overkill for initial deployment, Shuttle may not support
3. **Basic Auth** - REJECTED: Insecure over internet even with HTTPS

---

## Technology Choices Summary

| Component | Technology | Version | Justification |
|-----------|-----------|---------|---------------|
| SSE Server SDK | rmcp | 0.8.1 | Official MCP Rust SDK, production-ready, Shuttle-validated |
| HTTP Framework | axum | 0.8+ | Already in project, rmcp returns Axum router |
| Middleware | tower-http | 0.6+ | Existing dependency, optional CORS for browsers |
| Deployment | Shuttle.dev | N/A | Zero-config HTTPS, secrets management, Rust-native |
| Authentication (MVP) | None | N/A | HTTPS + secrets + network access control |
| Authentication (Prod) | OAuth2 PKCE | rmcp `auth` feature | Optional upgrade path for multi-tenant scenarios |

---

## Best Practices from rmcp Documentation

### SSE Server Lifecycle
```rust
// 1. Create SSE server with config
let (sse_server, router) = SseServer::new(config);

// 2. Register MCP service (tool handlers)
let ct = sse_server.with_service(BinanceServer::new);

// 3. Serve with graceful shutdown
let server = axum::serve(listener, router)
    .with_graceful_shutdown(async move {
        ct.cancelled().await;
    });
```

### Axum Integration Pattern
```rust
// Merge SSE routes with existing HTTP API
let app = Router::new()
    .merge(sse_router)  // /mcp/sse, /mcp/message
    .route("/health", get(health_check))  // Existing routes
    .route("/api/ticker/:symbol", get(ticker_api));  // Existing HTTP API
```

### Shuttle Deployment Pattern
```rust
#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleAxum {
    // Load Binance API keys from Shuttle secrets
    let api_key = secrets.get("BINANCE_API_KEY").unwrap();
    let api_secret = secrets.get("BINANCE_API_SECRET").unwrap();

    // Create SSE server + Axum router
    let router = create_sse_server(api_key, api_secret).await?;

    Ok(router.into())
}
```

---

## Architecture Decisions

### AD-001: Single Feature Flag Strategy

**Decision**: Use single `sse` feature flag that implies `http-api` feature

**Rationale**:
- SSE requires HTTP server (no stdio-only SSE deployment)
- Cargo.toml: `sse = ["http-api", "rmcp/transport-sse-server"]`
- Users enable SSE via `--features sse` (automatically enables http-api)

**Rejected Alternative**: Separate `sse` and `http-api` features - REJECTED: SSE cannot work without HTTP layer

---

### AD-002: Reuse Existing Server Architecture

**Decision**: SSE transport layer wraps existing `BinanceServer` without modifications

**Rationale**:
- `BinanceServer` already implements `rmcp::ServerHandler` trait
- SSE server calls same tool handlers as stdio transport
- Zero code duplication - DRY principle
- Constitution Principle III (Modular Architecture) compliance

**Implementation**:
```rust
// src/server/mod.rs - NO CHANGES
#[tool_handler]
impl ServerHandler for BinanceServer { /* existing code */ }

// src/transport/sse.rs - NEW
pub async fn create_sse_transport() -> SseServer {
    let (sse_server, router) = SseServer::new(config);
    sse_server.with_service(BinanceServer::new);  // Reuse existing server
    (sse_server, router)
}
```

---

### AD-003: No State Management Required

**Decision**: SSE connections are stateless - no session persistence

**Rationale**:
- Each SSE connection is independent
- Binance API calls are stateless (REST + WebSocket)
- Order book state already managed in-memory per symbol
- No need for Redis/PostgreSQL session storage

**Implication**: Server restarts require clients to reconnect (acceptable per spec SC-006: < 10s reconnection time)

---

## Dependencies Update

**New Dependencies** (add to Cargo.toml):
```toml
[dependencies]
rmcp = { version = "0.8.1", features = ["server", "transport-sse-server"] }
# Note: "auth" feature NOT included in MVP

[dependencies.shuttle-runtime]
version = "0.56.0"  # Latest Shuttle runtime

[dependencies.shuttle-axum]
version = "0.56.0"  # Shuttle Axum integration
```

**Feature Definition**:
```toml
[features]
sse = ["http-api", "rmcp/transport-sse-server"]
```

---

## Risk Analysis

| Risk | Severity | Mitigation |
|------|----------|-----------|
| rmcp API breaking changes | Low | Pin to 0.8.1, monitor upstream releases |
| Shuttle deployment failures | Medium | Test deployment in Shuttle staging environment first |
| SSE connection limits | Low | Spec requires 50 concurrent connections, Shuttle likely supports 100+ |
| OAuth complexity creep | Medium | Defer OAuth to Phase 2, document upgrade path |
| Binance rate limits shared across SSE clients | Medium | Existing GCRA limiter already handles this |

---

## Next Steps (Phase 1)

1. Create `src/transport/sse.rs` module with `SseServer` wrapper
2. Add SSE routes to `src/http/server.rs` via `Router::merge()`
3. Create `Shuttle.toml` with project configuration
4. Add `shuttle-runtime` and `shuttle-axum` dependencies
5. Update `main.rs` to support `--mode sse` flag
6. Write integration tests for SSE handshake and tool calls
7. Update CLAUDE.md with new dependencies

---

**Research Complete**: All NEEDS CLARIFICATION items resolved. Ready for Phase 1 design artifacts.
