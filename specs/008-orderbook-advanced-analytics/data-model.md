# Data Model: Advanced Order Book Analytics

**Feature**: 008-orderbook-advanced-analytics
**Date**: 2025-01-18
**Source**: Extracted from [spec.md](./spec.md) Key Entities (lines 104-113)

---

## Entity Overview

This feature introduces 5 new data types for orderbook analytics, all built on top of existing `OrderBook` types from Feature 007.

```
OrderBook (Feature 007)
    ↓
OrderFlowSnapshot → captures bid/ask pressure over time windows
VolumeProfile → histogram of traded volume across price levels
MarketMicrostructureAnomaly → detected abnormal market behavior
LiquidityVacuum → price ranges with abnormally low volume
AbsorptionEvent → large orders absorbing market pressure
```

---

## Core Entities

### 1. OrderFlowSnapshot

**Purpose**: Represents order flow state over a configurable time window (spec FR-001).

**Fields**:
| Field | Type | Description | Validation | Source |
|-------|------|-------------|------------|--------|
| `symbol` | `String` | Trading pair (e.g., "BTCUSDT") | Must match `^[A-Z]{4,12}$` | User input |
| `time_window_start` | `DateTime<Utc>` | Window start timestamp | Must be < `time_window_end` | Calculated |
| `time_window_end` | `DateTime<Utc>` | Window end timestamp | Must be ≤ now | Calculated |
| `window_duration_secs` | `u32` | Duration in seconds (10, 30, 60, 300) | One of: 10, 30, 60, 300 | Spec FR-001 |
| `bid_flow_rate` | `f64` | Bid orders per second | ≥ 0.0 | Aggregated count / duration |
| `ask_flow_rate` | `f64` | Ask orders per second | ≥ 0.0 | Aggregated count / duration |
| `net_flow` | `f64` | Bid flow - ask flow | Can be negative | `bid_flow_rate - ask_flow_rate` |
| `flow_direction` | `FlowDirection` | Enum: STRONG_BUY, MODERATE_BUY, NEUTRAL, MODERATE_SELL, STRONG_SELL | See FlowDirection rules | Spec FR-006 |
| `cumulative_delta` | `f64` | Running sum of (buy volume - sell volume) | Can be negative | Spec FR-007 |

**Relationships**:
- One `OrderFlowSnapshot` per (symbol, time window) tuple
- Derived from multiple `OrderBook` snapshots (aggregated over time_window)

**State Transitions**: None (immutable after creation)

**Lifecycle**: Created on-demand when user requests order flow metrics, not persisted (computed from time-series snapshots).

---

### 2. VolumeProfile

**Purpose**: Histogram of traded volume across price levels (spec FR-002).

**Fields**:
| Field | Type | Description | Validation | Source |
|-------|------|-------------|------------|--------|
| `symbol` | `String` | Trading pair | Must match `^[A-Z]{4,12}$` | User input |
| `time_period_start` | `DateTime<Utc>` | Analysis period start | Must be < `time_period_end` | User input |
| `time_period_end` | `DateTime<Utc>` | Analysis period end | Must be ≤ now | User input |
| `price_range_low` | `Decimal` | Lowest price in histogram | > 0 | Min from trades |
| `price_range_high` | `Decimal` | Highest price in histogram | > `price_range_low` | Max from trades |
| `bin_size` | `Decimal` | Price bin width (adaptive tick-based) | > 0 | `max(tick_size × 10, price_range / 100)` (Clarification Q1, Research) |
| `bin_count` | `usize` | Number of bins in histogram | 1-200 | `(price_range_high - price_range_low) / bin_size` |
| `histogram` | `Vec<VolumeBin>` | Volume distribution | Length = `bin_count` | Aggregated from @aggTrade |
| `total_volume` | `Decimal` | Sum of all bin volumes | ≥ 0 | Sum of `histogram[*].volume` |
| `point_of_control` | `Decimal` | Price level with highest volume (POC) | Within price range | Bin with max volume |
| `value_area_high` | `Decimal` | Upper boundary of value area (VAH) | ≥ POC | 70% volume upper bound |
| `value_area_low` | `Decimal` | Lower boundary of value area (VAL) | ≤ POC | 70% volume lower bound |

**Nested Type: VolumeBin**:
```rust
struct VolumeBin {
    price_level: Decimal,     // Center price of bin
    volume: Decimal,           // Total volume traded at this level
    trade_count: u64,          // Number of trades in bin
}
```

**Relationships**:
- One `VolumeProfile` per (symbol, time_period) request
- Derived from @aggTrade WebSocket data (clarification Q2)

**State Transitions**: None (immutable after creation)

**Lifecycle**: Created on-demand, may be cached for recently requested periods (<5 minutes old).

---

### 3. MarketMicrostructureAnomaly

**Purpose**: Detected abnormal market behavior (spec FR-003, FR-004, FR-005).

**Fields**:
| Field | Type | Description | Validation | Source |
|-------|------|-------------|------------|--------|
| `anomaly_id` | `Uuid` | Unique identifier | UUID v4 | Generated |
| `symbol` | `String` | Trading pair | Must match `^[A-Z]{4,12}$` | Monitoring |
| `anomaly_type` | `AnomalyType` | Enum: QuoteStuffing, IcebergOrder, FlashCrashRisk | See AnomalyType rules | Detection algorithm |
| `detection_timestamp` | `DateTime<Utc>` | When anomaly detected | ≤ now | System time |
| `confidence_score` | `f64` | Detection confidence (0.0-1.0) | 0.0 ≤ x ≤ 1.0 | Statistical threshold |
| `affected_price_levels` | `Vec<Decimal>` | Price levels involved | Non-empty for iceberg/flash crash | Detection context |
| `severity` | `Severity` | Enum: Low, Medium, High, Critical | Based on confidence + type | Combined heuristic |
| `recommended_action` | `String` | Human-readable advice | 1-100 chars | Template by anomaly type |
| `metadata` | `serde_json::Value` | Type-specific details | Valid JSON | Extra context (update rate, refill count, etc.) |

**Nested Type: AnomalyType**:
```rust
enum AnomalyType {
    QuoteStuffing {
        update_rate: f64,        // Updates/sec (>500 triggers, FR-003)
        fill_rate: f64,          // Percentage (<10% triggers)
    },
    IcebergOrder {
        price_level: Decimal,
        refill_rate_multiplier: f64,  // >5x median triggers (FR-004)
        median_refill_rate: f64,
    },
    FlashCrashRisk {
        depth_loss_pct: f64,     // >80% triggers (FR-005)
        spread_multiplier: f64,  // >10x average triggers
        cancellation_rate: f64,  // >90% triggers
    },
}
```

**Relationships**:
- Multiple anomalies can exist concurrently for same symbol
- No parent/child relationships

**State Transitions**:
```
Detected → [if confidence drops below threshold] → Resolved
Detected → [persists for >60 seconds] → Escalated (severity increases)
```

**Lifecycle**: Created when detected, stored for historical analysis (7-day retention aligned with snapshots).

---

### 4. LiquidityVacuum

**Purpose**: Price range with abnormally low volume (spec FR-008).

**Fields**:
| Field | Type | Description | Validation | Source |
|-------|------|-------------|------------|--------|
| `vacuum_id` | `Uuid` | Unique identifier | UUID v4 | Generated |
| `symbol` | `String` | Trading pair | Must match `^[A-Z]{4,12}$` | Monitoring |
| `price_range_low` | `Decimal` | Lower boundary | > 0 | Detection algorithm |
| `price_range_high` | `Decimal` | Upper boundary | > `price_range_low` | Detection algorithm |
| `volume_deficit_pct` | `f64` | Volume <20% of median (triggers detection) | 0.0-100.0 | `(median - actual) / median × 100` |
| `median_volume` | `Decimal` | Median volume for comparison | > 0 | Calculated from surrounding levels |
| `actual_volume` | `Decimal` | Volume in vacuum range | ≥ 0 | From volume profile |
| `expected_impact` | `ImpactLevel` | Enum: FastMovement, ModerateMovement, Negligible | Based on deficit % | Spec line 89 |
| `detection_timestamp` | `DateTime<Utc>` | When detected | ≤ now | System time |

**Relationships**:
- Derived from `VolumeProfile` histogram analysis
- One-to-many with price levels (vacuum spans multiple bins)

**State Transitions**: None (immutable)

**Lifecycle**: Created during volume profile generation, expires when profile regenerated.

---

### 5. AbsorptionEvent

**Purpose**: Large order absorbing market pressure without price movement (spec FR-009).

**Fields**:
| Field | Type | Description | Validation | Source |
|-------|------|-------------|------------|--------|
| `event_id` | `Uuid` | Unique identifier | UUID v4 | Generated |
| `symbol` | `String` | Trading pair | Must match `^[A-Z]{4,12}$` | Monitoring |
| `price_level` | `Decimal` | Exact price of absorption | > 0 | OrderBook update |
| `absorbed_volume` | `Decimal` | Cumulative volume absorbed | > 0 | Sum of refills |
| `refill_count` | `u32` | Number of refills observed | ≥ 1 | Counter |
| `first_detected` | `DateTime<Utc>` | First refill timestamp | ≤ `last_updated` | System time |
| `last_updated` | `DateTime<Utc>` | Most recent refill | ≤ now | System time |
| `suspected_entity_type` | `EntityType` | Enum: MarketMaker, Whale, Unknown | Heuristic | Refill pattern analysis |
| `direction` | `Direction` | Enum: Accumulation (bid absorption), Distribution (ask absorption) | Based on side | OrderBook side |

**Nested Type: EntityType**:
```rust
enum EntityType {
    MarketMaker,  // Refill rate 1-3x median (spec assumption line 132)
    Whale,        // Refill rate >5x median (iceberg-like)
    Unknown,      // Insufficient data
}
```

**Relationships**:
- One `AbsorptionEvent` per (symbol, price_level, direction) tuple
- Updated incrementally as new refills observed

**State Transitions**:
```
Detected (refill_count = 1)
    ↓
Active (refill_count > 1, absorbing pressure)
    ↓
Completed (price moved away OR >5 min no refills)
```

**Lifecycle**: Created on first refill, updated on subsequent refills, archived when price level no longer active.

---

## Supporting Enums

### FlowDirection (FR-006)
```rust
enum FlowDirection {
    STRONG_BUY,     // bid_flow > 2.0 × ask_flow
    MODERATE_BUY,   // bid_flow 1.2-2.0 × ask_flow
    NEUTRAL,        // bid_flow ≈ ask_flow (0.8-1.2 ratio)
    MODERATE_SELL,  // ask_flow 1.2-2.0 × bid_flow
    STRONG_SELL,    // ask_flow > 2.0 × bid_flow
}
```

### Severity
```rust
enum Severity {
    Low,       // Confidence 0.5-0.7
    Medium,    // Confidence 0.7-0.85
    High,      // Confidence 0.85-0.95
    Critical,  // Confidence >0.95 (triggers alert)
}
```

### ImpactLevel
```rust
enum ImpactLevel {
    FastMovement,      // Deficit >80%, expect >2% rapid move
    ModerateMovement,  // Deficit 50-80%, expect 1-2% move
    Negligible,        // Deficit <50%, minimal impact
}
```

### Direction
```rust
enum Direction {
    Accumulation,   // Bid-side absorption (buying pressure absorbed)
    Distribution,   // Ask-side absorption (selling pressure absorbed)
}
```

---

## Data Flow Diagrams

### Order Flow Calculation
```
WebSocket OrderBook Updates (Feature 007)
    ↓
RocksDB Snapshots (1/sec per symbol)
    ↓
Query Historical Snapshots (time window)
    ↓
Aggregate bid/ask order counts
    ↓
Calculate flow rates = counts / duration
    ↓
OrderFlowSnapshot {bid_flow_rate, ask_flow_rate, net_flow, flow_direction}
```

### Volume Profile Generation
```
@aggTrade WebSocket Stream (clarification Q2)
    ↓
Filter trades by time_period
    ↓
Group by price bins (adaptive tick-based, clarification Q1)
    ↓
Sum volume per bin → VolumeBin[]
    ↓
Find POC (max volume bin)
    ↓
Calculate VAH/VAL (70% volume boundaries)
    ↓
VolumeProfile {histogram, POC, VAH, VAL}
```

### Anomaly Detection Pipeline
```
OrderBook WebSocket Updates
    ↓
[Quote Stuffing Detector] → update_rate >500/sec + fill_rate <10% → MarketMicrostructureAnomaly
    ↓
[Iceberg Detector] → refill_rate >5x median → MarketMicrostructureAnomaly
    ↓
[Flash Crash Detector] → depth_loss >80% OR spread >10x OR cancellation_rate >90% → MarketMicrostructureAnomaly
    ↓
Store anomalies (7-day retention)
```

---

## Storage Schema

### RocksDB Time-Series Keys
```
Key Format: "{symbol}:{unix_timestamp_sec}"
Example: "BTCUSDT:1737158400"

Value: MessagePack-serialized OrderBookSnapshot {
    bids: Vec<(price, quantity)>,  // Top 20 levels
    asks: Vec<(price, quantity)>,  // Top 20 levels
    update_id: u64,
    timestamp: i64,
}

Index: Prefix scan by symbol for time-range queries
Query Example: Scan "BTCUSDT:1737158400" to "BTCUSDT:1737158460" (60-second window)
```

### Retention Policy
- Background task runs hourly
- Deletes keys with `unix_timestamp_sec < (now - 7 days)`
- Estimated cleanup: ~1.7M keys/day (86,400 snapshots/day/pair × 20 pairs)

---

## Validation Rules Summary

1. **Symbol Validation**: All symbols must match `^[A-Z]{4,12}$` pattern (alphanumeric uppercase, 4-12 chars)
2. **Time Windows**: Must be one of [10s, 30s, 60s, 300s] for order flow (spec FR-001)
3. **Timestamp Ordering**: `start < end ≤ now` for all time-based queries
4. **Volume Profile Bins**: Adaptive sizing `max(tick_size × 10, price_range / 100)` yields 50-200 bins
5. **Confidence Thresholds**: Only flag anomalies with confidence >0.95 (spec edge case line 68)
6. **Flow Direction**: Calculated from bid/ask flow ratio per FR-006 thresholds

---

## Performance Considerations

- **OrderFlowSnapshot**: <100ms calculation (NFR line 174) - achieved via RocksDB prefix scan (<10ms) + in-memory aggregation
- **VolumeProfile**: <500ms for 24h data (SC-002) - requires caching @aggTrade stream, not querying historical trades
- **Anomaly Detection**: Real-time streaming (no batch processing), <1ms per orderbook update
- **Storage Growth**: ~500MB-1GB for 7 days (12M snapshots × ~50 bytes compressed)

---

## Error Handling

| Error Condition | Response | HTTP Code (if applicable) |
|----------------|----------|--------------------------|
| Invalid symbol | "Symbol must be 4-12 uppercase letters" | 400 |
| Invalid time window | "time_window must be 10, 30, 60, or 300 seconds" | 400 |
| Time range too large | "Maximum time range: 7 days (snapshot retention limit)" | 400 |
| Insufficient data | "Insufficient data for volume profile (<100 trades)" | 422 |
| RocksDB query timeout | "Historical data query timeout (>200ms)" | 504 |

---

## Next Steps

After data model approval:
1. Generate JSON Schema contracts for 5 MCP tools (Phase 1 contracts)
2. Create quickstart.md with example requests/responses (Phase 1)
3. Update CLAUDE.md with new dependencies (Phase 1 agent context)
4. Generate implementation tasks (Phase 2 - `/speckit.tasks` command)
