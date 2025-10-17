# Data Model: Order Book Depth Tools

**Feature**: 007-orderbook-depth-tools
**Date**: 2025-10-17
**Source**: Extracted from spec.md Key Entities section

## Overview

This document defines the data structures for order book depth tracking, metrics calculation, and MCP tool responses. All entities use `rust_decimal::Decimal` for price/quantity precision (FR-017) and `i64` millisecond timestamps (FR-018).

---

## Core Entities

### OrderBook

**Purpose**: Represents real-time order book state for a single symbol, maintained via WebSocket delta updates or REST snapshots.

**Fields**:
- `symbol: String` - Trading pair (e.g., "BTCUSDT"), uppercased and validated
- `bids: BTreeMap<Decimal, Decimal>` - Price → quantity map, sorted descending (best bid first)
- `asks: BTreeMap<Decimal, Decimal>` - Price → quantity map, sorted ascending (best ask first)
- `last_update_id: i64` - Binance lastUpdateId for delta synchronization
- `timestamp: i64` - Last update time in milliseconds since Unix epoch

**Validation Rules**:
- Symbol must exist in Binance spot trading pairs (FR-012)
- Bids and asks must not overlap (best_bid < best_ask)
- Quantity must be > 0 (zero quantity = level deletion)
- BTreeMap automatically maintains sort order

**State Transitions**:
```
Uninitialized → Snapshot Loaded → Delta Updates → Stale (>5s) → Refresh
                                                  ↓
                                           WebSocket Disconnect → Reconnecting
```

**Relationships**:
- One OrderBook per symbol (1:1)
- Contained in OrderBookManager's HashMap<String, OrderBook> (N:1)
- Source for OrderBookMetrics computation (1:1)
- Source for OrderBookDepth serialization (1:1)

---

### OrderBookMetrics

**Purpose**: L1 aggregated metrics computed on-demand from OrderBook for progressive disclosure (FR-001, FR-006 to FR-010).

**Fields**:
- `symbol: String` - Trading pair
- `timestamp: i64` - Calculation time in milliseconds since Unix epoch
- `spread_bps: f64` - Spread in basis points: `((best_ask - best_bid) / best_bid) * 10000`
- `microprice: f64` - Volume-weighted fair price: `(best_bid * ask_vol + best_ask * bid_vol) / (bid_vol + ask_vol)`
- `bid_volume: f64` - Sum of top 20 bid level quantities
- `ask_volume: f64` - Sum of top 20 ask level quantities
- `imbalance_ratio: f64` - Bid/ask volume ratio: `bid_volume / ask_volume`
- `best_bid: Decimal` - Highest bid price
- `best_ask: Decimal` - Lowest ask price
- `walls: Vec<Wall>` - Significant price levels (qty > 2x median of top 20)
- `slippage_estimates: SlippageEstimates` - VWAP slippage for 10k, 25k, 50k USD

**Validation Rules**:
- All f64 fields must be finite (not NaN or infinity)
- spread_bps ≥ 0
- imbalance_ratio > 0
- best_bid < best_ask
- Timestamps must be within last 10 seconds (staleness check)

**Calculation Dependencies**:
- Requires OrderBook with ≥1 bid and ≥1 ask level
- Walls calculation requires ≥5 levels for median (returns empty Vec<Wall> otherwise)
- Slippage estimates require sufficient liquidity for target USD amounts

**Relationships**:
- Computed from OrderBook (1:1)
- Contains Vec<Wall> (1:N)
- Contains SlippageEstimates (1:1)

---

### Wall

**Purpose**: Represents a significant price level with quantity > 2x median of top 20 levels, useful for identifying support/resistance zones.

**Fields**:
- `price: Decimal` - Price level
- `qty: Decimal` - Quantity at this level
- `side: WallSide` - Enum: Bid | Ask

**Validation Rules**:
- qty > 2 * median(top 20 levels in same side)
- price must exist in OrderBook at time of detection

**Identification Logic**:
```rust
let top_20: Vec<Decimal> = bids.iter().take(20).map(|(_, q)| *q).collect();
let median = calculate_median(&top_20);
let threshold = median * Decimal::from(2);
for (price, qty) in bids {
    if qty > threshold {
        walls.push(Wall { price, qty, side: WallSide::Bid });
    }
}
```

**Relationships**:
- Multiple walls per OrderBookMetrics (N:1)
- Derived from OrderBook bid/ask levels

---

### SlippageEstimate

**Purpose**: VWAP-based slippage calculation for a target USD amount, enabling traders to estimate execution cost (FR-010).

**Fields**:
- `target_usd: f64` - Target amount in USD (10000, 25000, or 50000)
- `avg_price: f64` - Volume-weighted average price for this fill
- `slippage_bps: f64` - Slippage in basis points: `((avg_price - best_price) / best_price) * 10000`
- `filled_qty: f64` - Actual quantity filled (may be less than target if liquidity insufficient)
- `filled_usd: f64` - Actual USD amount filled

**Validation Rules**:
- target_usd ∈ {10000, 25000, 50000}
- filled_qty ≥ 0
- filled_usd ≥ 0 and ≤ target_usd (with tolerance for rounding)
- slippage_bps ≥ 0

**VWAP Calculation**:
```rust
let mut remaining_usd = target_usd;
let mut total_cost = 0.0;
let mut total_qty = 0.0;
for (price, qty) in asks.iter() {
    let max_qty_at_level = remaining_usd / price;
    let filled_at_level = qty.min(max_qty_at_level);
    total_cost += filled_at_level * price;
    total_qty += filled_at_level;
    remaining_usd -= filled_at_level * price;
    if remaining_usd <= 0.0 { break; }
}
let avg_price = total_cost / total_qty;
```

**Relationships**:
- Three instances per OrderBookMetrics (buy/sell for each target)
- Derived from OrderBook ask/bid levels

---

### SlippageEstimates

**Purpose**: Container for slippage estimates across all target amounts and directions.

**Fields**:
- `buy_10k_usd: Option<SlippageEstimate>` - None if insufficient ask liquidity
- `buy_25k_usd: Option<SlippageEstimate>`
- `buy_50k_usd: Option<SlippageEstimate>`
- `sell_10k_usd: Option<SlippageEstimate>` - None if insufficient bid liquidity
- `sell_25k_usd: Option<SlippageEstimate>`
- `sell_50k_usd: Option<SlippageEstimate>`

**Validation Rules**:
- At least one estimate must be Some (total order book not empty)
- Higher target amounts may be None if liquidity exhausted

**Relationships**:
- One per OrderBookMetrics (1:1)
- Contains up to 6 SlippageEstimate instances

---

### OrderBookDepth

**Purpose**: Compact integer representation of order book levels for L2-lite (20 levels) and L2-full (100 levels) responses, reducing JSON size by ~40% (FR-004).

**Fields**:
- `symbol: String` - Trading pair
- `timestamp: i64` - Snapshot time in milliseconds since Unix epoch
- `price_scale: i32` - Scaling factor for prices (fixed at 100)
- `qty_scale: i32` - Scaling factor for quantities (fixed at 100000)
- `bids: Vec<[i64; 2]>` - Array of [scaled_price, scaled_qty] tuples, sorted descending
- `asks: Vec<[i64; 2]>` - Array of [scaled_price, scaled_qty] tuples, sorted ascending

**Scaling Rules**:
- `scaled_price = (actual_price * price_scale).round() as i64`
- `scaled_qty = (actual_qty * qty_scale).round() as i64`
- Reverse: `actual_price = scaled_price as f64 / price_scale as f64`
- Reverse: `actual_qty = scaled_qty as f64 / qty_scale as f64`

**Example**:
```json
{
  "symbol": "BTCUSDT",
  "timestamp": 1699999999123,
  "price_scale": 100,
  "qty_scale": 100000,
  "bids": [[6765050, 123400], [6765000, 80000]],
  "asks": [[6765100, 98700], [6765150, 40000]]
}
```
Decodes to:
- Bid: 67650.50 @ 1.23400 BTC
- Bid: 67650.00 @ 0.80000 BTC
- Ask: 67651.00 @ 0.98700 BTC
- Ask: 67651.50 @ 0.40000 BTC

**Validation Rules**:
- bids.len() ≤ 100 (FR-002)
- asks.len() ≤ 100
- price_scale == 100 (fixed)
- qty_scale == 100000 (fixed)
- Bids sorted descending, asks sorted ascending

**Relationships**:
- Serialized from OrderBook (1:1)
- Returned by get_orderbook_depth tool (1:1)

---

### OrderBookHealth

**Purpose**: Service health status for operational monitoring (FR-003).

**Fields**:
- `status: HealthStatus` - Enum: ok | degraded | error
- `orderbook_symbols_active: usize` - Count of symbols with active WebSocket connections
- `last_update_age_ms: i64` - Milliseconds since last successful depth update (any symbol)
- `websocket_connected: bool` - Overall WebSocket health (true if ≥1 connection active)
- `timestamp: i64` - Health check time in milliseconds since Unix epoch
- `reason: Option<String>` - Human-readable error message if status != ok

**Status Rules**:
- `ok`: All connections healthy, last_update_age_ms <5000
- `degraded`: ≥1 connection active but some disconnected, or staleness >5s
- `error`: All connections down or critical failure

**Validation Rules**:
- orderbook_symbols_active ≤ 20 (FR-021)
- last_update_age_ms ≥ 0
- reason must be Some if status != ok

**Relationships**:
- Aggregated from OrderBookManager state (1:1)
- Returned by get_orderbook_health tool (1:1)

---

## Enums

### WallSide
```rust
pub enum WallSide {
    Bid,  // Support level (large buy order)
    Ask,  // Resistance level (large sell order)
}
```

### HealthStatus
```rust
pub enum HealthStatus {
    #[serde(rename = "ok")]
    Ok,          // All systems operational
    #[serde(rename = "degraded")]
    Degraded,    // Partial functionality
    #[serde(rename = "error")]
    Error,       // Critical failure
}
```

---

## Storage Layer

### OrderBookManager

**Purpose**: Top-level manager for all order book instances, enforces 20-symbol limit, handles lazy initialization.

**Fields**:
- `orderbooks: HashMap<String, OrderBook>` - Symbol → OrderBook map
- `websocket_clients: HashMap<String, WebSocketClient>` - Symbol → WebSocket connection map
- `rate_limiter: Arc<RateLimiter>` - Shared rate limiter for REST API calls
- `binance_client: BinanceClient` - REST API client for snapshots and fallback

**Operations**:
- `async fn get_or_initialize(&mut self, symbol: &str) -> Result<&OrderBook>` - Lazy init (FR-026)
- `async fn close_symbol(&mut self, symbol: &str) -> Result<()>` - Free capacity (FR-022)
- `fn health(&self) -> OrderBookHealth` - Aggregate health status (FR-003)
- `fn active_symbols(&self) -> Vec<String>` - List tracked symbols

**Constraints**:
- `orderbooks.len() ≤ 20` (FR-021)
- Automatic cleanup: Close LRU symbol if limit reached

---

## Index Structures

### BTreeMap vs HashMap Trade-offs

**BTreeMap (chosen for order book)**:
- ✅ Sorted iteration (required for best_bid/best_ask)
- ✅ Range queries (top N levels)
- ✅ O(log n) insert/remove
- ❌ Slightly slower than HashMap for point lookups

**HashMap (chosen for symbol tracking)**:
- ✅ O(1) symbol lookup
- ✅ Unordered iteration acceptable
- ❌ Cannot maintain sort order

---

## Serialization Format

All entities implement:
- `Serialize` (via serde) for MCP tool responses
- `Deserialize` (via serde) for testing
- `JsonSchema` (via schemars) for MCP tool discovery

Example OrderBookMetrics JSON:
```json
{
  "symbol": "BTCUSDT",
  "timestamp": 1699999999123,
  "spread_bps": 0.49,
  "microprice": 67650.31,
  "bid_volume": 25.73,
  "ask_volume": 21.12,
  "imbalance_ratio": 1.218,
  "best_bid": "67650.00",
  "best_ask": "67650.50",
  "walls": {
    "bids": [{"price": "67600.00", "qty": "5.5", "side": "Bid"}],
    "asks": [{"price": "67750.00", "qty": "6.2", "side": "Ask"}]
  },
  "slippage_estimates": {
    "buy_10k_usd": {"target_usd": 10000, "avg_price": 67662.1, "slippage_bps": 1.8, "filled_qty": 0.1478, "filled_usd": 10000.0},
    "sell_10k_usd": {"target_usd": 10000, "avg_price": 67638.0, "slippage_bps": 1.9, "filled_qty": 0.1478, "filled_usd": 10000.0}
  }
}
```

---

## Data Flow Diagram

```
Binance WebSocket          Binance REST API
       ↓                          ↓
 WebSocketClient    →    RateLimiter
       ↓                          ↓
   OrderBook    ←────────    BinanceClient
       ↓
OrderBookManager
       ↓
   ┌──────────────────────────────┐
   │  OrderBookMetrics (L1)       │  → get_orderbook_metrics
   │  OrderBookDepth (L2)         │  → get_orderbook_depth
   │  OrderBookHealth             │  → get_orderbook_health
   └──────────────────────────────┘
              ↓
         MCP Response
```

---

**Data Model Status**: ✅ **COMPLETE** - Ready for contract generation
