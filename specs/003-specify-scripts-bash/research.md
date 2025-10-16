# Research: HTTP REST API + WebSocket для Binance

**Date**: 2025-10-16
**Feature**: 003-http-websocket-api

## Summary

Conducted research on technology stack for adding HTTP REST API and WebSocket streams to MCP Binance Server. Key decisions: axum 0.8+ for HTTP framework, tokio-tungstenite 0.28+ for WebSocket client, simple API key authentication with in-memory token store, tower middleware for rate limiting/CORS, and tokio::sync::broadcast for fan-out to multiple subscribers.

---

## Decision 1: HTTP Framework

### **Decision: axum 0.8.6**

**Rationale:**
- **Type safety**: 100% safe Rust (`#![forbid(unsafe_code)]`), compile-time route verification
- **Tower integration**: Native tower::Service, seamless middleware from tower-http ecosystem
- **Tokio-native**: Built by Tokio team, deep integration with tokio 1.48+ runtime
- **Developer experience**: Intuitive API, excellent error handling, type-safe extractors
- **Performance**: Near-identical to actix-web in benchmarks, most efficient memory usage
- **Active development**: Latest version 0.8.6 (2025), strong community momentum
- **Production proven**: Used by Discord, Microsoft, and other large-scale systems

**Alternatives Considered:**
- **actix-web 4.11**: Mature, high performance, but steeper learning curve, custom middleware system, less ergonomic than axum
- **warp 0.3**: Functional filter-based design, but lower maintenance activity (last release April 2024), creator now recommends axum

**Version to Use:**
```toml
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors", "limit"] }
```

**References:**
- axum docs: https://docs.rs/axum/latest/axum/
- axum GitHub: https://github.com/tokio-rs/axum
- tower middleware: https://docs.rs/tower/latest/tower/
- Official announcement: https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0

**Integration Pattern:**
```rust
use axum::{
    routing::{get, post, delete},
    Router,
    extract::{State, Query, Json},
};
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
};

let app = Router::new()
    .route("/api/v1/ticker/price", get(get_ticker_price))
    .route("/api/v1/order", post(create_order).delete(cancel_order))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .with_state(app_state);
```

---

## Decision 2: WebSocket Client Architecture

### **Decision: tokio-tungstenite 0.28.0 + custom reconnection**

**Rationale:**
- **Industry standard**: Most mature WebSocket library (7.1 trust score), RFC6455 compliant
- **Tokio integration**: Native async/await support with tokio 1.48+ runtime
- **Production proven**: Battle-tested with Binance streams by multiple projects
- **Stream splitting**: Separate read/write tasks for concurrent operations
- **Auto ping/pong**: Built-in heartbeat handling
- **Flexibility**: Custom reconnection logic provides full control over retry strategy

**Alternatives Considered:**
- **stream-tungstenite 0.5**: Built-in reconnection wrapper, but newer/less adopted, adds abstraction layer
- **async-tungstenite**: Superseded by tokio-tungstenite for Tokio-based projects

**Version to Use:**
```toml
tokio-tungstenite = { version = "0.28", features = ["native-tls-vendored"] }
futures = "0.3"
```

**Fan-Out Pattern: tokio::sync::broadcast**

**Decision**: Use `tokio::sync::broadcast::channel` for distributing Binance market data to multiple WebSocket clients.

**Rationale:**
- **Multi-consumer**: Each sent value seen by all subscribers (perfect for market data)
- **Automatic slow receiver handling**: Drops oldest messages when buffer full, no backpressure
- **Lag detection**: Receivers can detect missed messages via `RecvError::Lagged(n)`
- **Zero-copy**: Arc-based message sharing, minimal memory overhead
- **Built-in**: Part of tokio, no additional dependencies

**Why NOT tokio::sync::mpsc:**
- Single consumer only, cannot broadcast to multiple clients
- Would require manual fan-out logic with per-client channels

**Architecture:**
```rust
// Binance WebSocket → broadcast channel → multiple client WebSockets
let (tx, _rx) = broadcast::channel(1000); // 1000 message buffer

// Binance reader task
tokio::spawn(async move {
    while let Some(msg) = binance_stream.next().await {
        let _ = tx.send(msg); // Broadcast to all subscribers
    }
});

// Each client gets their own receiver
let mut client_rx = tx.subscribe();
tokio::spawn(async move {
    while let Ok(msg) = client_rx.recv().await {
        client_websocket.send(msg).await;
    }
});
```

**Reconnection Strategy: Exponential Backoff**

**Pattern**: 100ms → 200ms → 400ms → ... → max 30s, with jitter

**Rationale:**
- Binance WebSocket disconnects every 30-60 minutes (expected behavior)
- Exponential backoff prevents thundering herd on reconnect
- Max 30s delay prevents excessive wait times
- Jitter (±25% random) distributes reconnection load

**Implementation:**
```rust
async fn connect_with_retry(url: &str) -> Result<WebSocketStream> {
    let mut delay = Duration::from_millis(100);
    let max_delay = Duration::from_secs(30);
    let max_attempts = 10;

    for attempt in 1..=max_attempts {
        match connect_async(url).await {
            Ok((stream, _)) => {
                tracing::info!("Connected to Binance WebSocket");
                return Ok(stream);
            }
            Err(e) => {
                tracing::warn!("Connection attempt {} failed: {}", attempt, e);
                if attempt < max_attempts {
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, max_delay);
                } else {
                    return Err(e.into());
                }
            }
        }
    }
    unreachable!()
}
```

**References:**
- tokio-tungstenite: https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/
- Tokio broadcast: https://tokio.rs/tokio/tutorial/channels
- Binance WebSocket API: https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams
- Building Binance WS clients (Medium, Sep 2024): https://medium.com/@ekfqlwcjswl/building-real-time-binance-websocket-clients-in-rust

---

## Decision 3: Authentication Strategy

### **Decision: Simple API Key (Bearer Token) Authentication**

**Rationale:**
- **Industry standard**: Financial APIs (Stripe, Twilio, Binance) use API keys for simplicity
- **MCP compliance**: stdio transport assumes local trust (per MCP spec), HTTP transport requires auth
- **Revocability**: Individual tokens can be revoked without global key rotation
- **Zero dependencies**: Implement with existing `Arc<RwLock<HashMap>>` pattern
- **Constitution compliant**: Environment variables only, no hardcoded secrets

**Why NOT JWT:**
- Unnecessary complexity for single-server scenario
- Adds dependency (jsonwebtoken crate)
- Token signing/verification overhead for no benefit in this use case
- API keys are simpler for clients to use and manage

**Two-Level Authentication:**

**Level 1: Client → MCP Server (API Key)**
```rust
// src/http/middleware/auth.rs
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TokenStore {
    tokens: Arc<RwLock<HashMap<String, TokenMetadata>>>,
}

pub async fn validate_bearer_token(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // Skip "Bearer "

    // Validate token (check in TokenStore)
    if !is_valid_token(token).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
```

**Level 2: MCP Server → Binance (HMAC SHA256)**
- Already implemented in `src/binance/client.rs`
- Uses existing `Credentials::from_env()` from feature 001
- No changes needed

**Token Storage:**
- In-memory `HashMap<String, TokenMetadata>` wrapped in `Arc<RwLock<>>`
- Loaded from environment variable: `MCP_AUTH_TOKENS="token1,token2,token3"`
- Optional: Load from file for more complex scenarios (future enhancement)

**Security Considerations:**
- Tokens validated on every request (no session state)
- Failed validations logged at WARN level (Constitution compliance)
- No sensitive data in error responses (return generic "Unauthorized")
- Tokens stored as plain strings (not hashed) since they're opaque API keys

**Version to Use:**
```toml
# No additional dependencies needed
# Uses existing: tokio, serde, thiserror
```

**References:**
- MCP Security Spec: https://spec.modelcontextprotocol.io/specification/2024-11-05/security/
- tower-http auth: https://docs.rs/tower-http/latest/tower_http/auth/
- Constitution Security-First: `.specify/memory/constitution.md` § I

---

## Decision 4: Rate Limiting Implementation

### **Decision: tower::limit::RateLimitLayer**

**Rationale:**
- **Tower-native**: Seamless integration with axum middleware stack
- **Request-level limiting**: Prevents abuse of HTTP endpoints
- **Simple configuration**: Rate and burst parameters
- **Zero-copy**: Efficient async semaphore-based implementation
- **Constitution compliant**: Respects Binance weight system via existing client

**Pattern:**
```rust
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use std::time::Duration;

let middleware = ServiceBuilder::new()
    .layer(RateLimitLayer::new(100, Duration::from_secs(60))) // 100 req/min
    .layer(TraceLayer::new_for_http());

let app = Router::new()
    .route("/api/v1/ticker/price", get(handler))
    .layer(middleware);
```

**Binance Rate Limit Integration:**
- Existing `BinanceClient` already handles Binance weight limits (feature 001)
- HTTP API rate limit is for client protection, not Binance limit enforcement
- Binance 429 errors still propagate to HTTP clients with Retry-After header

**Version to Use:**
```toml
tower = { version = "0.5", features = ["limit"] }
```

**Alternative Considered:**
- **governor crate**: More features (per-IP, per-user), but overkill for MVP
- **Custom semaphore**: Reinventing the wheel, tower::limit is battle-tested

---

## Decision 5: OpenAPI Documentation

### **Decision: Manual OpenAPI 3.1 YAML (with future utoipa consideration)**

**Rationale:**
- **API-first design**: OpenAPI spec documents hand-written axum routes
- **Type safety preserved**: axum's compile-time checking > codegen
- **Simplicity**: No bidirectional sync complexity
- **Constitution acceptable**: Manual "glue logic" allowed per Constitution II

**Why NOT code generation:**
- OpenAPI → Rust codegen (openapi-generator, progenitor) adds:
  - Additional build step complexity
  - Dependency on codegen tool
  - Loss of axum's ergonomic extractors
  - Bidirectional sync burden (code ↔ spec)
- axum already provides type-safe routing, no need for generated types

**Future Enhancement: utoipa**
- Consider `utoipa` crate for auto-generating OpenAPI from axum routes
- Requires proc macros on handlers, adds some boilerplate
- Evaluate in Phase 2 if manual YAML maintenance becomes burden

**Specification Format:**
```yaml
openapi: 3.1.0
info:
  title: Binance MCP HTTP API
  version: 1.0.0
servers:
  - url: http://localhost:3000
security:
  - bearerAuth: []
paths:
  /api/v1/ticker/price:
    get:
      summary: Get current price
      # ... full spec
```

**Documentation Hosting:**
- Serve swagger-ui via axum static files (optional)
- Or use external swagger-editor for spec validation

**Version to Use:**
```toml
# No dependencies for Phase 1 (manual YAML)
# Optional future:
# utoipa = { version = "4.2", features = ["axum"] }
# utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

**References:**
- OpenAPI 3.1 Spec: https://spec.openapis.org/oas/v3.1.0
- utoipa docs: https://docs.rs/utoipa/latest/utoipa/

---

## Decision 6: Testing Strategy

### **Decision: Multi-layer testing with reqwest + wiremock**

**Layers:**

1. **Unit Tests**: Individual handler functions with mock state
2. **Integration Tests**: Full axum Router with real BinanceClient (mocked Binance API)
3. **End-to-End Tests**: Against Binance testnet or mock server

**Tools:**

**reqwest 0.12.24** (already in project)
- HTTP client for integration tests
- Test REST endpoints as a real client would

**wiremock 0.6**
- Mock Binance API responses in CI
- Record/replay for deterministic tests
- Verify request signatures, headers, parameters

**tungstenite 0.28** (non-tokio version)
- WebSocket client for testing WS streams
- Connect to server's /ws/* endpoints in tests

**Example Integration Test:**
```rust
#[tokio::test]
async fn test_get_ticker_price() {
    // Setup mock Binance API
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/24hr"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"symbol":"BTCUSDT","price":"45000.50"})))
        .mount(&mock_server)
        .await;

    // Start test server
    let app = create_test_app(&mock_server.uri());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, app));

    // Test request
    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/api/v1/ticker/price?symbol=BTCUSDT", addr))
        .header("Authorization", "Bearer test_token")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["price"], "45000.50");
}
```

**Version to Use:**
```toml
[dev-dependencies]
reqwest = "0.12.24"  # Already in project
wiremock = "0.6"
tungstenite = "0.28"  # Non-tokio for sync tests
```

**References:**
- wiremock: https://docs.rs/wiremock/latest/wiremock/
- axum testing: https://github.com/tokio-rs/axum/tree/main/examples/testing

---

## Summary of Decisions

| Component | Technology | Version | Rationale |
|-----------|-----------|---------|-----------|
| **HTTP Framework** | axum | 0.8 | Type safety, tower integration, tokio-native |
| **WebSocket Client** | tokio-tungstenite | 0.28 | Industry standard, tokio integration, proven |
| **Fan-Out** | tokio::sync::broadcast | Built-in | Multi-consumer, zero-copy, lag detection |
| **Authentication** | API Key (Bearer) | Custom | Simple, revocable, MCP compliant |
| **Rate Limiting** | tower::limit | 0.5 | Tower-native, simple, efficient |
| **API Docs** | OpenAPI 3.1 YAML | Manual | API-first, type-safe, simple |
| **Testing** | reqwest + wiremock | 0.12 / 0.6 | Multi-layer, deterministic, CI-friendly |

**Total New Dependencies**: 4 (axum, tokio-tungstenite, tower, tower-http) + 1 dev (wiremock)

**Constitution Compliance**: ✅ All 7 principles satisfied (see plan.md § Constitution Check)

**Next Phase**: Create data-model.md and contracts/ (Phase 1)
