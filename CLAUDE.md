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
- 2025-10-17: Completed 007-orderbook-depth-tools implementation (34/43 tasks)
  - All 3 user stories implemented: Quick Spread Assessment (L1), Detailed Depth Analysis (L2), Service Health Monitoring
  - Progressive disclosure strategy: L1 (15% token cost) → L2-lite (50%, 20 levels) → L2-full (100%, 100 levels)
  - Lazy initialization with WebSocket streaming (<symbol>@depth@100ms)
  - REST API fallback when data stale (>5s)
  - 20 concurrent symbol limit with GCRA rate limiting (1000 req/min)
- 007-orderbook-depth-tools: Added Rust 1.90+ (Edition 2024)
- 004-specify-scripts-bash: Added Rust 1.90+ (Edition 2024)

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
