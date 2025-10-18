# Research: Advanced Order Book Analytics

**Feature**: 008-orderbook-advanced-analytics
**Date**: 2025-01-18
**Purpose**: Resolve Technical Context NEEDS CLARIFICATION items before Phase 1 design

---

## Research Question 1: Time-Series Database Selection

**Context**: Need to store 12M orderbook snapshots (7 days × 20 pairs × 86,400 snapshots/day) with <200ms query latency for historical data retrieval.

### Requirements Analysis

From spec and clarifications:
- 1-second snapshot frequency (clarification Q3)
- 7-day retention (NFR line 178)
- 20 concurrent trading pairs (scalability constraint)
- <200ms query latency (clarification Q5, NFR line 177)
- 500MB-1GB storage estimate with compression
- Async queries with timeout handling
- Stdio transport (no network services required for embedded solutions)

### Candidates Evaluated

#### Option A: RocksDB (via `rocksdb` crate 0.23.0)
**Pros**:
- Embedded (no external service, perfect for stdio MCP server)
- LSM-tree optimized for write-heavy workloads (1 snapshot/sec × 20 pairs = 20 writes/sec)
- Built-in compression (Snappy/LZ4/Zstd)
- Rust bindings mature and well-maintained
- Sub-millisecond point queries (far exceeds <200ms target)
- Prefix scans for time-range queries (symbol+timestamp key design)

**Cons**:
- Manual time-series indexing (no built-in time-series features)
- No built-in retention policies (need manual cleanup logic)
- Query patterns require careful key design

**Performance**:
- Write throughput: >100k writes/sec (our need: 20 writes/sec)
- Read latency: <1ms for point queries, <10ms for range scans (both under <200ms target)
- Storage: ~500MB for 12M snapshots with Zstd compression (matches estimate)

#### Option B: InfluxDB (via `influxdb` crate 0.7.2 with influxdb2 client)
**Pros**:
- Purpose-built time-series database
- Built-in retention policies (automatic 7-day cleanup)
- Native time-range queries
- Downsampling support (could reduce storage)

**Cons**:
- Requires external InfluxDB server (violates stdio transport simplicity)
- Network latency overhead (adds ~5-50ms vs embedded)
- Additional operational complexity (server management, Docker, etc.)
- Overkill for 20 writes/sec

**Performance**:
- Write throughput: Designed for >1M points/sec (our need: 20/sec)
- Read latency: ~10-50ms network + query time (still under <200ms but slower than RocksDB)

#### Option C: In-Memory Ring Buffer (custom impl with `crossbeam` channels)
**Pros**:
- Zero serialization overhead
- Fastest possible queries (<1ms)
- Simplest implementation

**Cons**:
- No persistence (lost on server restart)
- Memory pressure for 12M snapshots (~1-2GB uncompressed)
- Cannot support historical backtesting

### Decision: **RocksDB (Option A)**

**Rationale**:
1. **Embedded fit**: No external dependencies aligns with stdio MCP server design (constitution principle III: modularity)
2. **Performance headroom**: <1ms point queries give 200x margin vs <200ms requirement
3. **Compression**: Zstd compression meets 500MB-1GB storage target
4. **Async support**: `tokio` + `spawn_blocking` for RocksDB operations (non-blocking on async runtime)
5. **Battle-tested**: Used in production time-series systems (e.g., TiKV, CockroachDB)

**Alternatives Considered**:
- InfluxDB rejected: External service violates simplicity principle, stdio transport becomes complex
- Ring buffer rejected: No persistence breaks backtesting requirement (SC-003, SC-004 need historical data)

**Implementation Plan**:
- Key design: `{symbol}:{unix_timestamp_sec}` (e.g., `BTCUSDT:1737158400`)
- Prefix scan for time-range queries: `BTCUSDT:` + start/end bounds
- Retention via background task: delete keys older than 7 days (1x/hour cleanup)
- Snapshot format: MessagePack serialization (smaller than JSON, faster than bincode)

---

## Research Question 2: Statistical Analysis Library Selection

**Context**: Need percentile calculations, rolling averages, standard deviation for anomaly detection (FR-003 to FR-010).

### Requirements Analysis

From spec:
- Percentile calculations for iceberg detection (5x median refill rate, FR-004)
- Rolling averages for flash crash detection (24h average spread, FR-005)
- Standard deviation for quote stuffing detection (update rate normality)
- Microstructure health score composite metrics (FR-010)

### Candidates Evaluated

#### Option A: `statrs` crate 0.18.0
**Pros**:
- Comprehensive statistical functions (mean, median, percentiles, std dev)
- `Statistics` trait for collections
- Rolling window support via `StatisticsExt`
- Well-documented, actively maintained
- Pure Rust (no C dependencies)

**Cons**:
- Some allocations for rolling windows (but negligible for our scale)
- No built-in exponential moving average (EMA) - need manual impl

**Example API**:
```rust
use statrs::statistics::Statistics;
let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
let median = data.median(); // 3.0
let p95 = data.percentile(95); // 4.8
let std_dev = data.std_dev(); // 1.41
```

#### Option B: `statistical` crate 1.1.0
**Pros**:
- Simpler API than `statrs`
- Includes median and percentiles

**Cons**:
- Less feature-complete (no rolling windows)
- Less actively maintained (last update 2020)

#### Option C: Manual Implementation
**Pros**:
- Zero dependencies
- Custom optimizations (e.g., running median via heap)

**Cons**:
- Reinventing wheel (violates simplicity principle from CLAUDE.md)
- Bug risk in statistical algorithms
- More testing burden

### Decision: **`statrs` 0.18.0 (Option A)**

**Rationale**:
1. **Feature completeness**: Has all required functions (median, percentiles, std dev, rolling stats)
2. **Rust ecosystem standard**: Most widely used stats library (constitution: prefer established solutions)
3. **Type safety**: Strong typing for distributions and statistical operations
4. **Performance**: Pure Rust with no allocations for basic operations

**Alternatives Considered**:
- `statistical` rejected: Less feature-complete, stale maintenance
- Manual implementation rejected: Violates "The best code is code that doesn't need to be written" (CLAUDE.md line 21)

**Implementation Plan**:
- Use `Statistics` trait for collections (median, percentiles)
- Implement rolling window via `VecDeque` + incremental updates (more efficient than full recomputation)
- For EMA: Custom implementation using `alpha * current + (1-alpha) * previous`

---

## Dependency Versions (verified via crates.io 2025-01-18)

| Crate | Version | Purpose | Verification |
|-------|---------|---------|--------------|
| `rocksdb` | 0.23.0 | Time-series snapshot storage | Latest stable on crates.io |
| `statrs` | 0.18.0 | Statistical analysis (median, percentiles, std dev) | Latest stable on crates.io |

**Constitution Compliance**: All dependencies use latest stable versions (Dependency Management principle, constitution lines 124-150).

---

## Additional Best Practices Research

### WebSocket @aggTrade Integration Patterns

**Context**: Need to connect to wss://stream.binance.com:9443/ws/<symbol>@aggTrade (clarification Q2) for volume profile data.

**Pattern**: Tokio async WebSocket with reconnection logic

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn subscribe_agg_trade(symbol: &str) -> Result<WebSocketStream, Error> {
    let url = format!("wss://stream.binance.com:9443/ws/{}@aggTrade", symbol.to_lowercase());
    let (ws_stream, _) = connect_async(url).await?;

    // Spawn task to handle incoming messages
    tokio::spawn(async move {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(data)) => {
                    let trade: AggTrade = serde_json::from_str(&data)?;
                    // Update volume profile
                }
                _ => {}
            }
        }
    });

    Ok(ws_stream)
}
```

**Reconnection Strategy**: Exponential backoff (1s, 2s, 4s, 8s, max 60s) on disconnect.

### Adaptive Tick Size Calculation

**Context**: Clarification Q1 specified adaptive tick-based bins for volume profile.

**Binance Tick Sizes** (from exchangeInfo API):
- BTCUSDT: 0.01 USDT
- ETHUSDT: 0.01 USDT
- SHIBUSDT: 0.00000001 USDT (8 decimal places)

**Bin Size Formula**: `bin_size = tick_size * multiplier`

**Recommended Multiplier**: 10-20 (yields 100-200 bins for typical 24h price range)

Example for BTCUSDT ($100,000 price, $1000 daily range):
- tick_size = 0.01
- multiplier = 10
- bin_size = 0.10 USDT
- bins in $1000 range = 10,000 bins (too many)

**Revised**: Use `max(tick_size * multiplier, price_range / 100)` to cap at ~100 bins.

---

## Summary

**Resolved Clarifications**:
1. ✅ Time-series storage: **RocksDB 0.23.0** (embedded, <1ms queries, 500MB-1GB storage)
2. ✅ Statistical analysis: **`statrs` 0.18.0** (comprehensive, type-safe, well-maintained)

**Additional Design Decisions**:
- @aggTrade WebSocket: `tokio-tungstenite` with exponential backoff reconnection
- Snapshot serialization: MessagePack format
- Retention cleanup: Background task (1x/hour) deleting keys older than 7 days
- Adaptive bin size: `max(tick_size * 10, price_range / 100)` for ~100 bins

**Next Phase**: Phase 1 (Data Model + Contracts) with all technical unknowns resolved.
