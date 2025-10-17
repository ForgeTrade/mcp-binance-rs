# Feature Specification: Order Book Depth Tools

**Feature Branch**: `007-orderbook-depth-tools`
**Created**: 2025-10-17
**Status**: Draft
**Input**: User description: "Add order book depth analysis tools to MCP server with progressive disclosure strategy (L1→L2-lite→L2-full) for optimal token economy. Tools: orderbook.metrics (L1 aggregates), orderbook.get (L2 depth), orderbook.health (service check). WebSocket + Local L2 Cache architecture for sub-100ms latency. Compact integer scaling to reduce JSON size by 40%."

## Clarifications

### Session 2025-10-17

- Q: What is the maximum number of concurrent symbols the system should support for order book tracking? → A: Up to 20 concurrent symbols
- Q: How should the system handle Binance API rate limits (1200 requests/minute REST API)? → A: Client-side rate limiter (1000 req/min, queue excess requests)
- Q: How are symbols initially subscribed to order book tracking? → A: Lazy initialization (subscribe on first request per symbol)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quick Spread Assessment (Priority: P1)

A trader wants to quickly assess if the current spread and liquidity conditions are favorable for entering a position without consuming excessive tokens or waiting for full order book data.

**Why this priority**: This is the most common use case and delivers immediate value with minimal token cost. 90% of trading decisions can be made from L1 metrics alone (spread, microprice, imbalance). This is the foundation for progressive disclosure.

**Independent Test**: Can be fully tested by calling `get_orderbook_metrics` with a symbol and verifying metrics are returned within 200ms. Delivers immediate value for spread-checking and quick liquidity assessment without requiring L2 data.

**Acceptance Scenarios**:

1. **Given** user has BTCUSDT as active symbol, **When** requesting `get_orderbook_metrics`, **Then** system returns spread_bps, microprice, bid/ask volumes, imbalance_ratio, best_bid/ask within 200ms (first request may take 2-3s for lazy initialization)
1a. **Given** symbol not yet tracked, **When** requesting metrics for first time, **Then** system initializes WebSocket subscription, fetches snapshot, then returns metrics (latency 2-3s for first request only)
2. **Given** order book has significant walls, **When** requesting metrics, **Then** system identifies walls (qty > 2x median of top 20 levels) in both bids and asks
3. **Given** user wants slippage estimate, **When** requesting metrics with default targets (10k, 25k, 50k USD), **Then** system returns VWAP-based slippage estimates for both buy and sell directions
4. **Given** WebSocket connection is down, **When** requesting metrics, **Then** system falls back to REST API with 2-5s latency and logs warning

---

### User Story 2 - Detailed Depth Analysis (Priority: P2)

A trader needs to inspect actual price levels beyond best bid/ask to identify support/resistance zones, calculate custom slippage, or analyze market microstructure. They should be able to progressively request more detail (L2-lite → L2-full) based on initial findings.

**Why this priority**: This serves advanced analysis needs but is not always required. By making it P2, we ensure L1 metrics are sufficient for most cases, and L2 is only fetched when analysis requires actual depth levels.

**Independent Test**: Can be tested independently by calling `get_orderbook_depth` with levels=20 (L2-lite) or levels=100 (L2-full) and verifying depth arrays are returned with compact integer scaling. Delivers value for advanced microstructure analysis.

**Acceptance Scenarios**:

1. **Given** user needs top 20 levels (L2-lite), **When** requesting `get_orderbook_depth` with levels=20, **Then** system returns compact integer arrays (price_scale=100, qty_scale=100000) with bids/asks within 300ms
2. **Given** user needs full depth (L2-full), **When** requesting levels=100, **Then** system returns all 100 bid/ask levels from local cache with same compact format
3. **Given** compact format is used, **When** parsing response, **Then** actual prices are decoded as scaled_price/100, quantities as scaled_qty/100000
4. **Given** local cache is stale (>5s), **When** requesting depth, **Then** system refreshes from REST API snapshot before returning data
5. **Given** symbol has low liquidity, **When** requesting levels=100, **Then** system returns all available levels even if fewer than 100 exist

---

### User Story 3 - Service Health Monitoring (Priority: P3)

User wants to verify that order book data is fresh and WebSocket connections are healthy before making trading decisions based on depth data.

**Why this priority**: This is an operational concern rather than core trading functionality. It's important for reliability but doesn't directly contribute to trading decisions. Most users will trust the data is fresh unless debugging issues.

**Independent Test**: Can be tested by calling `get_orderbook_health` and verifying status, active symbols count, last update age, and WebSocket status are returned. Delivers operational visibility.

**Acceptance Scenarios**:

1. **Given** WebSocket connections are active, **When** requesting health check, **Then** system returns status="ok", websocket_connected=true, last_update_age_ms<500
2. **Given** multiple symbols are being tracked, **When** requesting health, **Then** system returns count of orderbook_symbols_active
3. **Given** WebSocket disconnected, **When** requesting health, **Then** system returns status="degraded", websocket_connected=false, with reason

---

### Edge Cases

- **What happens when symbol doesn't exist?** System returns error with clear message "Symbol not found or not supported" (HTTP 404 equivalent in MCP)
- **What happens when WebSocket connection drops?** System automatically attempts reconnection (exponential backoff: 1s, 2s, 4s, 8s, max 30s), falls back to REST API for requests during downtime, logs warning
- **What happens when order book snapshot fails to load?** System retries up to 3 times with 1s delay, returns error if all attempts fail with message "Failed to load order book snapshot"
- **What happens when local cache is empty but WebSocket is connected?** System fetches initial snapshot via REST API, then switches to delta updates
- **What happens when quantity is zero (deleted level)?** System removes the price level from BTreeMap during delta processing
- **What happens with extreme imbalance (e.g., 100:1 ratio)?** Metrics report actual ratio, may indicate low liquidity or one-sided market (valid data, not error)
- **What happens when walls calculation has insufficient data (<20 levels)?** System uses available levels for median calculation, returns empty walls array if <5 levels available
- **What happens when slippage target exceeds available liquidity?** System calculates VWAP for available liquidity and reports actual filled amount vs target in response
- **What happens when symbol limit (20 concurrent) is reached?** System returns error "Maximum concurrent symbols (20) reached" and suggests closing unused symbols or prioritizing by activity
- **What happens when rate limit is exceeded?** Client-side rate limiter queues excess requests up to 30s, then returns error "Request queue full, rate limit exceeded" if queue capacity reached

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide `get_orderbook_metrics` tool returning L1 aggregated metrics (spread_bps, microprice, volumes, imbalance_ratio, walls, slippage_estimates) with P95 latency ≤200ms
- **FR-002**: System MUST provide `get_orderbook_depth` tool returning L2 depth data with configurable levels (default 20, max 100) in compact integer format with P95 latency ≤300ms
- **FR-003**: System MUST provide `get_orderbook_health` tool returning service status (ok/degraded/error), active symbol count, last update age, WebSocket connection status
- **FR-004**: System MUST use compact integer scaling (price_scale=100, qty_scale=100000) for L2 depth responses to reduce JSON size by ~40%
- **FR-005**: System MUST maintain local L2 cache using WebSocket depth stream (`<symbol>@depth@100ms`) with automatic snapshot fetch and delta updates
- **FR-006**: System MUST calculate spread in basis points as `((best_ask - best_bid) / best_bid) * 10000`
- **FR-007**: System MUST calculate microprice as `(best_bid * ask_vol + best_ask * bid_vol) / (bid_vol + ask_vol)` where volumes are top-of-book quantities
- **FR-008**: System MUST calculate imbalance ratio as `bid_volume / ask_volume` where volumes sum top 20 levels
- **FR-009**: System MUST identify walls as levels with quantity > 2x median quantity of top 20 levels
- **FR-010**: System MUST calculate slippage estimates for default targets (10k, 25k, 50k USD) using VWAP calculation
- **FR-011**: System MUST fall back to REST API (`GET /api/v3/depth?symbol=X&limit=100`) when WebSocket is unavailable, with degraded latency (2-5s)
- **FR-012**: System MUST validate symbol parameter against Binance spot trading pairs (case-insensitive, uppercase normalization)
- **FR-013**: System MUST handle WebSocket disconnections with exponential backoff reconnection (1s, 2s, 4s, 8s, max 30s between attempts)
- **FR-014**: System MUST process delta updates from WebSocket to maintain accurate local order book state
- **FR-015**: System MUST use `BTreeMap<Decimal, Decimal>` for order book storage to maintain sorted price levels
- **FR-016**: System MUST refresh local cache from REST API snapshot if last update age exceeds 5 seconds
- **FR-017**: System MUST use `rust_decimal` or equivalent for price/quantity precision to avoid floating-point errors
- **FR-018**: System MUST return timestamps in milliseconds since Unix epoch for all responses
- **FR-019**: System MUST log WebSocket connection status changes at INFO level
- **FR-020**: System MUST log cache refresh operations at DEBUG level
- **FR-021**: System MUST enforce maximum of 20 concurrent symbol subscriptions, returning error when limit reached
- **FR-022**: System MUST track active symbols and provide mechanism to close/unsubscribe unused symbols to free capacity
- **FR-023**: System MUST implement client-side rate limiter with 1000 requests/minute limit (conservative buffer below Binance 1200/min)
- **FR-024**: System MUST queue excess REST API requests for up to 30 seconds when rate limit reached, rejecting with error after queue timeout
- **FR-025**: System MUST log rate limit queue events at WARN level when queue depth exceeds 50% capacity
- **FR-026**: System MUST implement lazy initialization for symbol subscriptions, subscribing to WebSocket on first request for each symbol
- **FR-027**: System MUST accept first-request latency of 2-3s for lazy initialization (snapshot fetch + WebSocket connect), subsequent requests achieve <200ms target

### Key Entities

- **OrderBook**: Represents order book state for a single symbol. Key attributes: symbol (String), bids (BTreeMap<Decimal, Decimal> sorted descending), asks (BTreeMap<Decimal, Decimal> sorted ascending), last_update_id (i64), timestamp (i64 milliseconds since epoch). Updated via WebSocket deltas or REST snapshots.

- **OrderBookMetrics**: L1 aggregated metrics calculated from OrderBook. Key attributes: spread_bps (f64), microprice (f64), bid_volume (f64 sum of top 20), ask_volume (f64 sum of top 20), imbalance_ratio (f64), best_bid (Decimal), best_ask (Decimal), walls (Vec<Wall>), slippage_estimates (SlippageEstimates). Computed on-demand from current OrderBook state.

- **Wall**: Significant price level with large quantity. Key attributes: price (Decimal), qty (Decimal), side (Bid/Ask). Identified as levels with qty > 2x median of top 20 levels.

- **SlippageEstimate**: VWAP-based slippage calculation for target USD amount. Key attributes: target_usd (f64), avg_price (f64 VWAP), slippage_bps (f64), filled_qty (f64), filled_usd (f64). Separate estimates for buy and sell directions.

- **OrderBookDepth**: Compact integer representation of order book levels. Key attributes: symbol (String), timestamp (i64), price_scale (i32 = 100), qty_scale (i32 = 100000), bids (Vec<[i64; 2]> scaled integers), asks (Vec<[i64; 2]> scaled integers). Used for L2-lite (20 levels) and L2-full (100 levels) responses.

- **OrderBookHealth**: Service health status. Key attributes: status (ok/degraded/error), orderbook_symbols_active (usize count), last_update_age_ms (i64), websocket_connected (bool), timestamp (i64). Indicates operational health of order book tracking.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `get_orderbook_metrics` tool returns L1 metrics with P95 latency ≤200ms when WebSocket is connected
- **SC-002**: `get_orderbook_depth` tool returns L2 data with P95 latency ≤300ms for both L2-lite (20 levels) and L2-full (100 levels)
- **SC-003**: Compact integer scaling reduces JSON response size by ≥35% compared to full decimal representation
- **SC-004**: Token usage for L1 metrics is ≤15% of equivalent L2-full depth data token cost
- **SC-005**: Progressive disclosure strategy (L1→L2-lite→L2-full) achieves 35% overall token reduction in typical analysis workflows compared to always fetching L2-full
- **SC-006**: WebSocket connection maintains <5s staleness 99.9% of the time (last_update_age_ms <5000)
- **SC-007**: System successfully handles WebSocket disconnections and reconnects within 30 seconds in 99% of cases
- **SC-008**: Spread calculation accuracy is within 0.01 bps of manual calculation from raw order book data
- **SC-009**: Microprice calculation is within $0.01 of expected value for BTCUSDT (high-value pairs)
- **SC-010**: Slippage estimates are within 5% of actual execution slippage for default targets (10k, 25k, 50k USD)
- **SC-011**: Wall identification correctly flags levels with qty > 2x median in 95% of manual review cases
- **SC-012**: Order book depth data matches Binance REST API snapshot within 100ms of same timestamp
- **SC-013**: System logs all WebSocket reconnection events at INFO level for operational monitoring
- **SC-014**: REST API fallback activates within 1s of WebSocket disconnection detection
- **SC-015**: Local cache refresh from REST API completes within 2s for symbols with <1000 total levels
- **SC-016**: First request for new symbol (lazy initialization) completes within 3s for 95% of cases
- **SC-017**: Rate limiter successfully prevents Binance 418/429 errors in 99% of normal operation scenarios
