# Research: Order Book Depth Tools

**Feature**: 007-orderbook-depth-tools
**Date**: 2025-10-17
**Status**: Complete

## Unknowns Resolved

### 1. WebSocket Client: tokio-tungstenite

**Decision**: tokio-tungstenite 0.27.0

**Rationale**: Latest async WebSocket client with Tokio bindings, significantly more performant than previous versions, seamlessly integrates with tokio 1.48.0 async runtime for Binance depth streams (`<symbol>@depth@100ms`).

**Alternatives Considered**:
- `async-tungstenite`: Generic async WebSocket but requires manual executor binding
- `tokio-websockets`: Newer but less mature ecosystem support
- Rejected both in favor of tokio-tungstenite's proven stability and tokio-specific optimizations

**Compatibility**: Fully compatible with tokio 1.48.0; supports all standard WebSocket operations (ping/pong, binary/text messages, close frames) with async/await patterns.

---

### 2. Decimal Precision: rust_decimal

**Decision**: rust_decimal 1.37.2

**Rationale**: Industry-standard for financial precision with 96-bit mantissa avoiding floating-point errors, preserves trailing zeros and exact base-10 accuracy critical for order book price/quantity calculations. Required for FR-017 compliance.

**Alternatives Considered**:
- `bigdecimal`: Arbitrary precision but slower (heap allocations)
- `fixed`: Compile-time fixed-point but inflexible for varying decimal places
- Rejected both in favor of rust_decimal's stack-allocated 128-bit representation (96-bit mantissa + scale)

**Compatibility**: Pure Rust implementation with no async runtime dependencies; works seamlessly with any Rust project including tokio-based applications.

---

### 3. Rate Limiting: governor

**Decision**: governor 0.10.1

**Rationale**: Implements GCRA (Generic Cell Rate Algorithm, functionally equivalent to leaky bucket but 10x faster in multi-threaded scenarios), uses only 64-bit state with lock-free compare-and-swap operations, requires just 3 lines for async/await usage. Essential for FR-023, FR-024 (1000 req/min client-side rate limiting with 30s queue).

**Alternatives Considered**:
- `leaky-bucket`: Task-based coordination model with higher overhead
- `ratelimit_meter`: Deprecated in favor of governor
- Rejected leaky-bucket due to: slower performance in multi-threaded workloads, coordinator task complexity, higher memory overhead vs governor's 64-bit state

**Comparison - governor vs leaky-bucket**:
| Factor | governor | leaky-bucket |
|--------|----------|--------------|
| Performance | 10x faster in multi-threaded | Baseline |
| Memory | 64-bit state | Task coordination overhead |
| Async integration | Direct async/await | Coordinator task required |
| Ecosystem | Tower/Axum/Tonic/Hyper via tower-governor | Standalone |
| Algorithm | GCRA (continuous nanosecond updates) | Token bucket (discrete refills) |

**Compatibility**: Fully compatible with tokio 1.48.0 through async/await support; integrates with Tower middleware ecosystem (tower-governor) for broader framework compatibility; no background tasks required unlike traditional token bucket implementations.

---

### 4. Feature Flag Naming

**Decision**: `orderbook` feature flag

**Rationale**: Short, descriptive name matching module structure (`src/orderbook/`). Consistent with existing feature naming patterns in Binance MCP server (spot, margin, futures).

**Alternatives Considered**:
- `orderbook-depth`: Too verbose for common usage
- `depth-tools`: Less clear scope
- `ob`: Too cryptic
- Rejected verbose/cryptic options in favor of clear, concise `orderbook`

**Cargo.toml addition**:
```toml
[features]
default = ["orderbook"]
orderbook = ["tokio-tungstenite", "rust_decimal", "governor"]
```

---

## Technology Stack Summary

**Core Dependencies** (resolved):
- tokio-tungstenite 0.27.0 - WebSocket client
- rust_decimal 1.37.2 - Financial precision
- governor 0.10.1 - Rate limiting

**Existing Dependencies** (confirmed compatible):
- rmcp 0.8.1 - MCP Server SDK
- tokio 1.48.0 - Async runtime
- reqwest 0.12.24 - HTTP client (REST API fallback)
- serde 1.0.228 + serde_json 1.0.145 - Serialization
- schemars 1.0.4 - JSON Schema
- thiserror 2.0.17 - Error handling
- tracing 0.1.41 + tracing-subscriber 0.3.20 - Logging

**Total New Dependencies**: 3 (tokio-tungstenite, rust_decimal, governor)

---

## Best Practices Applied

### WebSocket Depth Streams (Binance API)
- Stream: `wss://stream.binance.com:9443/ws/<symbol>@depth@100ms`
- Initial snapshot: `GET /api/v3/depth?symbol=BTCUSDT&limit=100`
- Delta updates: Apply bids/asks changes to local BTreeMap
- Reconnection: Exponential backoff (1s, 2s, 4s, 8s, max 30s per FR-013)
- Staleness detection: Track last_update_id, refresh if >5s old (FR-016)

### Order Book State Management
- BTreeMap<Decimal, Decimal> for sorted price levels (bids descending, asks ascending)
- Remove price level when quantity = 0 (delta delete operation)
- Atomic updates: Apply all delta changes in single lock scope
- Symbol tracking: HashMap<String, OrderBook> with 20-symbol limit (FR-021)

### Rate Limiting Strategy
- GCRA parameters: 1000 permits/60 seconds = 16.67 permits/second burst
- Queue depth: 30s * 16.67 req/s ≈ 500 requests max queue size
- Rejection: Return error "Request queue full, rate limit exceeded" after 30s (FR-024)
- Logging: WARN when queue depth >50% capacity (FR-025)

### Metrics Calculations (FR-006 to FR-010)
- Spread (bps): `((best_ask - best_bid) / best_bid) * 10000`
- Microprice: `(best_bid * ask_vol + best_ask * bid_vol) / (bid_vol + ask_vol)`
- Imbalance: `sum(top 20 bid volumes) / sum(top 20 ask volumes)`
- Walls: Levels with `qty > 2 * median(top 20 levels)`
- Slippage: VWAP calculation for 10k, 25k, 50k USD targets

### Compact Integer Scaling (FR-004)
- price_scale = 100 → divide price by 100 to get actual value
- qty_scale = 100000 → divide quantity by 100000 to get actual value
- Example: BTCUSDT price 67650.50 → stored as 6765050 (i64)
- Example: Quantity 1.23456 → stored as 123456 (i64)
- Size reduction: ~40% vs full decimal JSON representation

---

## Integration Patterns

### Lazy Initialization Flow (FR-026, FR-027)
```
1. First request for symbol X arrives
2. Check if OrderBook exists for X
3. If not:
   a. Check if symbol limit (20) reached → error if yes
   b. Fetch REST snapshot: GET /api/v3/depth?symbol=X&limit=100
   c. Subscribe to WebSocket: wss://stream.binance.com:9443/ws/x@depth@100ms
   d. Initialize OrderBook with snapshot
   e. Add to HashMap<String, OrderBook>
4. Return metrics/depth from OrderBook
```

Latency: First request 2-3s (REST + WebSocket connect), subsequent <200ms (local cache)

### WebSocket Reconnection Flow (FR-013)
```
1. Detect disconnect (ping timeout, connection error)
2. Log at INFO level: "WebSocket disconnected for {symbol}"
3. Start exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s (max)
4. Between retries: Serve requests from stale cache + REST fallback
5. On reconnect: Fetch fresh snapshot, resume delta updates
6. Log at INFO level: "WebSocket reconnected for {symbol}"
```

### REST API Fallback Flow (FR-011)
```
1. WebSocket unavailable or stale (>5s)
2. Rate limiter: Check if 1000 req/min budget available
3. If yes: GET /api/v3/depth?symbol=X&limit=100 (2-5s latency)
4. If no: Queue request for up to 30s
5. After 30s: Return error "Request queue full, rate limit exceeded"
6. Log at WARN level if fallback used
```

---

## Risk Mitigation

### Memory Management
- **Risk**: 20 symbols * 100 levels * 2 sides * (16 bytes Decimal + 16 bytes Decimal) ≈ 128 KB
- **Mitigation**: Acceptable memory footprint; BTreeMap automatically manages heap allocation
- **Monitoring**: Track active_symbols count in health endpoint

### WebSocket Stability
- **Risk**: Connection drops during high volatility
- **Mitigation**: Exponential backoff reconnection + REST fallback + 5s staleness tolerance
- **Monitoring**: Log reconnection events at INFO level (FR-019)

### Rate Limit Compliance
- **Risk**: Binance IP ban (418 error) if rate limit exceeded
- **Mitigation**: Conservative 1000/min limit (buffer below 1200/min), 30s request queue
- **Monitoring**: Log queue depth at WARN when >50% capacity (FR-025)

### Decimal Precision
- **Risk**: Floating-point errors causing incorrect calculations
- **Mitigation**: rust_decimal 96-bit mantissa for exact base-10 representation
- **Monitoring**: Success criteria SC-008 to SC-011 verify calculation accuracy

---

## Testing Strategy

### Unit Tests
- Metrics calculations: Verify spread_bps, microprice, imbalance formulas (SC-008 to SC-010)
- Wall detection: Confirm 2x median threshold logic (SC-011)
- Compact scaling: Roundtrip encoding/decoding (SC-003)
- Symbol limit: Enforce 20 concurrent max (FR-021)

### Integration Tests
- WebSocket reconnection: Simulate disconnect → verify exponential backoff (SC-007)
- Rate limiter: Burst 1200 requests → verify 1000/min enforcement (SC-017)
- Lazy initialization: First request latency ≤3s (SC-016)
- REST fallback: WebSocket down → verify 2-5s fallback latency

### Performance Tests
- L1 metrics: P95 latency ≤200ms (SC-001)
- L2 depth: P95 latency ≤300ms (SC-002)
- Cache refresh: ≤2s for <1000 levels (SC-015)
- Staleness: <5s 99.9% of time (SC-006)

---

## Deployment Considerations

### Feature Flag Usage
```toml
# Cargo.toml
[features]
default = ["orderbook"]
orderbook = ["tokio-tungstenite", "rust_decimal", "governor"]
```

Enable: `cargo build --features orderbook` (default)
Disable: `cargo build --no-default-features --features spot,server`

### Environment Variables
- `BINANCE_API_KEY` - Optional (public depth API doesn't require auth)
- `BINANCE_SECRET_KEY` - Optional (not used for order book depth)
- `RUST_LOG=info` - Recommended to monitor WebSocket reconnections

### Resource Requirements
- Memory: ~128 KB for 20 symbols * 100 levels
- CPU: Negligible (delta updates are O(log n) BTreeMap inserts)
- Network: ~1-5 KB/s per symbol (WebSocket depth stream)
- Disk: None (in-memory only)

---

**Research Status**: ✅ **COMPLETE** - All unknowns resolved, ready for Phase 1 design
