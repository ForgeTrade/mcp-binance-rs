# mcp-binance-rs Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-17

## Active Technologies
- Rust 1.90+ (Edition 2024) - Updated from 1.75+ with 2021 edition
- rmcp 0.8.1 - MCP Server SDK with macros
- tokio 1.48.0 - Async runtime with full features
- axum 0.8+ - HTTP server framework (003-specify-scripts-bash)
- tokio-tungstenite 0.28.0 - WebSocket client for order book streaming
- tower 0.5+ - Middleware stack (003-specify-scripts-bash)
- tower-http 0.6+ - HTTP-specific middleware (003-specify-scripts-bash)
- governor 0.6 - GCRA rate limiting for Binance API
- rust_decimal 1.37.2 - Financial precision arithmetic (96-bit mantissa)
- reqwest 0.12.24 - HTTP client (json + rustls-tls)
- serde 1.0.228 + serde_json 1.0.145 - Serialization
- schemars 1.0.4 - JSON Schema generation
- chrono 0.4 - Date and time library with serde support
- thiserror 2.0.17 - Error handling
- tracing 0.1.41 + tracing-subscriber 0.3.20 - Logging
- N/A (tests use Binance Testnet API and in-memory mock servers) (004-specify-scripts-bash)
- In-memory (BTreeMap<Decimal, Decimal> for order book state, HashMap for symbol tracking) (007-orderbook-depth-tools)
- RocksDB embedded time-series database (1-second snapshots, 7-day retention, ~12M snapshots for 20 pairs, ~500MB-1GB with Zstd compression). Key design: `{symbol}:{unix_timestamp_sec}`, prefix scans for time-range queries. Background cleanup task deletes keys older than 7 days. (008-orderbook-advanced-analytics)
- N/A (SSE is stateless protocol, session state in-memory only) (009-specify-scripts-bash)
- Rust 1.90+ (Edition 2024) + axum 0.8+ (HTTP server), tokio 1.48.0 (async runtime), rmcp 0.8.1 (MCP SDK) (010-specify-scripts-bash)
- In-memory HashMap for session management (no changes) (010-specify-scripts-bash)

## Project Structure
```
src/
  binance/          # Binance API client
  config/           # Configuration management
  error/            # Error types
  http/             # HTTP API server (feature: http-api)
  orderbook/        # Order book depth tools (feature: orderbook)
    types.rs        # Core data structures (OrderBook, OrderBookMetrics, OrderBookDepth, OrderBookHealth)
    manager.rs      # OrderBookManager (lazy init, 20 symbol limit, WebSocket + REST)
    metrics.rs      # L1 metrics & L2 depth encoding
    websocket.rs    # WebSocket client (<symbol>@depth@100ms streams)
    rate_limiter.rs # GCRA rate limiter (1000 req/min, 30s queue)
    tools.rs        # MCP tool handlers
  server/           # MCP server core
  tools/            # MCP tool implementations
tests/
  integration/      # Integration tests
  unit/             # Unit tests
```

## Commands
cargo build --features orderbook
cargo run --features orderbook
cargo test --features orderbook
cargo clippy --features orderbook

## Code Style
Rust 1.75+ (Edition 2024): Follow standard conventions

## Dependency Management Policy
- ALL dependencies MUST be kept up-to-date with latest stable versions
- Version updates verified via context7 or crates.io API
- Security patches applied within 7 days
- Major version updates require CHANGELOG review
- See constitution.md § Dependency Management for full policy

## Recent Changes
- 2025-10-19: **Completed Feature 011: Mainnet Support with Secure API Key Authentication** (48/48 tasks, 100% ✅)
  - ✅ Session-scoped credential management (per-session API keys, no disk persistence)
  - ✅ Environment-aware routing (testnet.binance.vision ↔ api.binance.com)
  - ✅ Three credential tools: configure_credentials, get_credentials_status, revoke_credentials
  - ✅ Dual transport support with feature gates (#[cfg(feature = "sse")])
  - ✅ Security: API secrets never logged, credentials cleared on session end
  - ✅ Public tools always use mainnet regardless of configured credentials
  - ✅ Comprehensive error handling with 6 structured error codes (FR-009, FR-013)
  - ✅ Inline code documentation for locking strategies and security considerations
  - ✅ Quickstart scenarios validated for testnet credentials and error handling
  - Key pattern: `SessionManager::get_credentials() → BinanceClient::get_base_url(credentials)`
- 010-specify-scripts-bash: Added Rust 1.90+ (Edition 2024) + axum 0.8+ (HTTP server), tokio 1.48.0 (async runtime), rmcp 0.8.1 (MCP SDK)
- 2025-10-18: **Completed Feature 009: SSE Transport for Cloud Deployment** (54/65 tasks, 83%)
  - ✅ Phase 1-5 complete (100%): SSE transport, Shuttle.dev integration, dual transport support
  - ✅ Remote HTTPS access via Server-Sent Events (rmcp SDK)
  - ✅ Shuttle.dev deployment ready with automatic HTTPS and secrets management
  - ✅ Dual transport: stdio (local) + SSE (cloud) with feature flags
  - ✅ Session management (50 concurrent connections, 30s timeout)
  - ✅ Health monitoring endpoints (`/health`, `/mcp/sse`, `/mcp/message`)
  - ✅ Comprehensive documentation (deployment guide, troubleshooting, feature flags)
  - ⏸️ Deferred to production: Actual Shuttle deployment verification (T037-T038, T061-T062, T065)
  - 📝 MVP complete: Server is cloud-deployment-ready, all core functionality implemented
- 009-specify-scripts-bash: Added Rust 1.90+ (Edition 2024)
  - All 3 user stories implemented: Quick Spread Assessment (L1), Detailed Depth Analysis (L2), Service Health Monitoring
  - Progressive disclosure strategy: L1 (15% token cost) → L2-lite (50%, 20 levels) → L2-full (100%, 100 levels)
  - Lazy initialization with WebSocket streaming (<symbol>@depth@100ms)
  - REST API fallback when data stale (>5s)
  - 20 concurrent symbol limit with GCRA rate limiting (1000 req/min)

## Order Book Module Architecture (feature: orderbook)
### Progressive Disclosure Strategy
- **L1 Metrics** (15% token cost): Spread, microprice, imbalance, walls, slippage estimates
- **L2-lite** (50% token cost): 20 depth levels with compact integer encoding
- **L2-full** (100% token cost): 100 depth levels with compact integer encoding

### Key Features
- **Lazy Initialization**: WebSocket subscribes on first request (2-3s cold start, <200ms warm)
- **Symbol Limit**: Maximum 20 concurrent symbols tracked
- **Rate Limiting**: GCRA algorithm (1000 req/min, 30s queue timeout)
- **Data Freshness**: REST fallback when WebSocket data >5s old
- **Compact Encoding**: price_scale=100, qty_scale=100000 (40% JSON size reduction)

### MCP Tools
1. `get_orderbook_metrics` - L1 aggregated metrics for quick spread assessment
2. `get_orderbook_depth` - L2 depth with compact integer arrays (levels=1-100)
3. `get_orderbook_health` - Service health monitoring (status, active symbols, staleness)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
