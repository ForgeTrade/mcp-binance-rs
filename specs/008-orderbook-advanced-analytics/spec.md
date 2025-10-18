# Feature Specification: Advanced Order Book Analytics

**Feature Branch**: `008-orderbook-advanced-analytics`
**Created**: 2025-01-18
**Status**: Draft
**Input**: User description: "Advanced Order Book Analytics - Order Flow, Volume Profile, Market Microstructure"

## Clarifications

### Session 2025-01-18

- Q: Volume profile histogram requires binning traded volume across price levels. How should price levels be grouped into bins? → A: Adaptive tick-based bins (use exchange's native tick size × configurable multiplier)
- Q: Volume profile requires trade data from Binance. Which trade stream endpoint should be used? → A: @aggTrade (Aggregated trades endpoint)
- Q: Time-series storage will retain 7 days of orderbook snapshots for flow analysis. At what frequency should snapshots be captured? → A: 1 second (1 snapshot/sec)
- Q: Order flow analysis tracks bid/ask order additions and cancellations. Should the system track individual order IDs or just aggregated counts? → A: Aggregated counts only (orders/sec per side)
- Q: What is the maximum acceptable latency for querying historical orderbook snapshots from time-series storage? → A: <200ms for historical data retrieval

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Order Flow Analysis for Trade Timing (Priority: P1)

Algorithmic traders need to see **order flow dynamics** (rate of order additions/cancellations) to identify optimal entry and exit points based on buying/selling pressure changes over time.

**Why this priority**: Order flow is the most fundamental indicator of market sentiment and provides immediate trading signals. Without it, traders miss crucial momentum shifts.

**Independent Test**: Can be fully tested by requesting order flow metrics for BTCUSDT over the last 60 seconds and verifying that bid/ask flow rates are calculated correctly, delivering actionable "buy pressure increasing" or "sell pressure increasing" signals.

**Acceptance Scenarios**:

1. **Given** BTCUSDT orderbook WebSocket is connected, **When** trader requests order flow metrics for last 60 seconds, **Then** system returns bid flow rate (orders/sec), ask flow rate (orders/sec), net flow (bid - ask), and flow direction indicator (buying/selling pressure)

2. **Given** order flow shows high bid flow rate (>100 orders/min), **When** trader analyzes the data, **Then** system highlights this as "strong buying pressure" with timestamp of pressure surge

3. **Given** sudden spike in order cancellations on bid side, **When** orderbook updates, **Then** system detects "liquidity withdrawal" event and flags it in order flow analysis

---

### User Story 2 - Volume Profile for Support/Resistance Discovery (Priority: P2)

Technical analysts need to see **volume distribution across price levels** to identify high-volume nodes (HVN) and low-volume nodes (LVN) which act as support/resistance zones.

**Why this priority**: Volume profile reveals hidden support/resistance that price-only charts miss. Critical for swing traders and position sizing.

**Independent Test**: Can be tested by requesting volume profile for ETHUSDT over 24 hours and verifying that the system correctly identifies the price level with highest traded volume (Point of Control) and value area (70% of volume).

**Acceptance Scenarios**:

1. **Given** 24 hours of ETHUSDT trade data, **When** trader requests volume profile, **Then** system returns histogram showing volume distribution across price levels, Point of Control (POC), Value Area High (VAH), and Value Area Low (VAL)

2. **Given** volume profile shows POC at $3,500, **When** current price approaches $3,500, **Then** system highlights this as "high-volume support/resistance zone"

3. **Given** low-volume node (LVN) between $3,550-$3,580, **When** trader analyzes profile, **Then** system flags this as "liquidity vacuum - expect fast price movement through this zone"

---

### User Story 3 - Market Microstructure Anomaly Detection (Priority: P3)

Risk managers need to detect **market microstructure anomalies** (quote stuffing, iceberg orders, flash crashes) to avoid trading during manipulated or unstable market conditions.

**Why this priority**: Protects traders from executing during abnormal market conditions. Prevents losses from manipulation and ensures fair trading environment.

**Independent Test**: Can be tested by simulating quote stuffing scenario (>1000 order updates/second with no trades) and verifying system detects and flags this as "HFT manipulation detected".

**Acceptance Scenarios**:

1. **Given** normal market conditions (<100 orderbook updates/sec), **When** quote stuffing occurs (>500 updates/sec with <10% fill rate), **Then** system detects "quote stuffing - potential HFT manipulation" and recommends avoiding trades

2. **Given** large hidden iceberg order detected (bid level with >5x median refill rate), **When** trader analyzes orderbook, **Then** system highlights "suspected iceberg order at $106,400 - institutional accumulation likely"

3. **Given** sudden liquidity drain (>80% of top 20 levels disappear in <1 second), **When** flash crash risk emerges, **Then** system triggers "flash crash risk - extreme caution advised" alert

---

### Edge Cases

- What happens when WebSocket connection drops during order flow calculation? (System should mark data as stale and resume calculation on reconnection)
- How does system handle volume profile requests for low-liquidity pairs with <100 trades/day? (Return "insufficient data" with minimum threshold recommendation)
- What if iceberg detection triggers false positives on legitimate market maker activity? (Use statistical confidence threshold - only flag patterns >95% confidence)
- How to distinguish between natural spread widening and flash crash precursor? (Compare current spread vs 24h average - flag if >10x wider)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST calculate order flow metrics (bid flow rate, ask flow rate, net flow, flow direction) over configurable time windows (10s, 30s, 60s, 5min) using aggregated order counts per side (no individual order ID tracking required)

- **FR-002**: System MUST generate volume profile histogram showing volume distribution across price levels with Point of Control (POC), Value Area High (VAH), and Value Area Low (VAL). Price levels MUST be binned using adaptive tick-based strategy (exchange's native tick size × configurable multiplier) to maintain consistent granularity across different price ranges

- **FR-003**: System MUST detect quote stuffing by monitoring orderbook update rate (>500 updates/sec) vs trade fill rate (<10% fills) and flag when ratio exceeds threshold

- **FR-004**: System MUST identify suspected iceberg orders by tracking refill rate of individual price levels (>5x median refill rate = iceberg)

- **FR-005**: System MUST monitor for flash crash precursors: sudden liquidity drain (>80% depth loss in <1s), abnormal spread widening (>10x average), high order cancellation rate (>90%)

- **FR-006**: System MUST provide order flow direction indicators: "STRONG_BUY" (bid flow >2x ask flow), "MODERATE_BUY" (bid flow 1.2-2x ask flow), "NEUTRAL" (flow balanced), "MODERATE_SELL", "STRONG_SELL"

- **FR-007**: System MUST calculate cumulative delta: running sum of (buy volume - sell volume) to show net market aggression over time window

- **FR-008**: System MUST identify liquidity vacuums: price ranges where volume <20% of median volume (indicating potential for rapid price movement)

- **FR-009**: System MUST track absorption events: large orders at specific price levels that repeatedly absorb market pressure without price movement (whale accumulation/distribution)

- **FR-010**: System MUST provide microstructure health score (0-100): composite metric combining spread stability, liquidity depth, order flow balance, and update rate normality

### Key Entities

- **OrderFlowSnapshot**: Represents order flow state over a time window - includes bid/ask flow rates (aggregated counts per second), net flow, flow direction, timestamp range, symbol (no individual order ID tracking)

- **VolumeProfile**: Histogram of traded volume across price levels - includes POC (highest volume price), VAH/VAL (value area boundaries), total volume, price range, time period, bin size (adaptive tick-based: native tick size × multiplier), bin count

- **MarketMicrostructureAnomaly**: Detected abnormal market behavior - includes anomaly type (quote stuffing, iceberg, flash crash risk), confidence score, timestamp, affected price levels, recommended action

- **LiquidityVacuum**: Price range with abnormally low volume - includes price range, volume deficit percentage, expected impact (fast movement), detection timestamp

- **AbsorptionEvent**: Large order absorbing market pressure - includes price level, absorbed volume, refill count, suspected entity type (market maker, whale), accumulation/distribution direction

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Traders can identify order flow direction changes within 5 seconds of pressure shift occurring

- **SC-002**: Volume profile calculations complete in under 500ms for 24-hour data window on major pairs (BTCUSDT, ETHUSDT)

- **SC-003**: Quote stuffing detection achieves >95% precision (false positive rate <5%) on historical manipulation events

- **SC-004**: Iceberg order detection identifies >80% of confirmed institutional orders based on post-trade analysis

- **SC-005**: Flash crash risk alerts trigger at least 30 seconds before liquidity event occurs (early warning system)

- **SC-006**: Microstructure health score correlates >0.7 with subsequent 5-minute price volatility (predictive power validation)

- **SC-007**: System processes >1000 orderbook updates/second without dropping metrics calculations (performance under load)

- **SC-008**: Liquidity vacuum detection identifies >90% of price levels that experience >2% rapid movement in backtesting

## Assumptions *(mandatory)*

- WebSocket orderbook stream from Binance provides sufficient granularity for order flow analysis (assumes <100ms update latency)
- Volume profile calculations use aggregated trade stream (@aggTrade) for accurate volume distribution, which combines fills at same price/time and reduces data volume 60-80% vs raw trades
- Quote stuffing detection threshold (500 updates/sec, <10% fill rate) is calibrated for cryptocurrency markets (may need adjustment for traditional markets)
- Iceberg detection assumes market makers refill levels gradually (1-3x median), while icebergs refill aggressively (5x+)
- Flash crash detection requires comparing current metrics to 24-hour rolling averages (assumes sufficient historical data available)
- Traders understand microstructure concepts (order flow, POC, VAH/VAL, iceberg orders) - no educational UI required
- System focuses on spot markets initially (futures microstructure patterns may differ)

## Dependencies *(mandatory)*

- **Feature 007 (Order Book Depth Tools)**: Requires existing WebSocket orderbook infrastructure, OrderBook types, and real-time update mechanism
- **Binance Trade Stream API**: Needs access to aggregated trades via wss://stream.binance.com:9443/ws/<symbol>@aggTrade for volume profile calculations (distinct from orderbook depth)
- **Time-Series Storage**: Requires ability to store and query historical orderbook snapshots at 1-second intervals for flow analysis (7 days retention: ~12M snapshots for 20 pairs, ~500MB-1GB storage with compression) - not currently implemented
- **Statistical Analysis Library**: Needs percentile calculations, rolling averages, standard deviation for anomaly detection

## Scope *(mandatory)*

### In Scope
- Real-time order flow metrics (bid/ask flow rates, net flow, direction)
- Volume profile generation for configurable time windows (1h, 4h, 24h)
- Quote stuffing, iceberg order, and flash crash risk detection
- Liquidity vacuum and absorption event identification
- Microstructure health scoring

### Out of Scope
- Historical backtesting engine (metrics are real-time only)
- Automated trading signals or execution (analytics only, no trading automation)
- Cross-exchange microstructure comparison (single exchange focus)
- Machine learning-based anomaly detection (uses statistical thresholds)
- Visualization/charting (returns raw data, UI rendering is client responsibility)
- Futures-specific microstructure (funding rate impact, basis, open interest) - spot markets only

## Non-Functional Requirements *(optional)*

### Performance
- Order flow calculations must complete within 100ms of WebSocket update
- Volume profile generation must handle 100,000+ trades without memory overflow
- Anomaly detection must process 1000+ orderbook updates/second without lag
- Historical snapshot queries from time-series storage must complete in <200ms to support real-time flow analysis (<300ms total end-to-end latency)

### Scalability
- Support concurrent analysis of up to 20 trading pairs (same as orderbook feature limit)
- Time-series storage must retain 7 days of historical snapshots captured at 1-second intervals (86,400 snapshots/day per pair, 12M total for 20 pairs)

### Reliability
- Gracefully handle WebSocket disconnections without losing in-progress calculations
- Provide data staleness indicators when metrics are calculated from outdated snapshots

## Constitution Compliance *(mandatory)*

- **Security-First**: ✅ No new authentication requirements (uses existing Binance connection)
- **Auto-Generation Priority**: ✅ No code generation required (manual analytics implementation)
- **Modular Architecture**: ✅ New feature will be feature-gated: `orderbook_analytics` separate from base `orderbook`
- **Type Safety**: ✅ All metrics (OrderFlowSnapshot, VolumeProfile, etc.) will use strong typing with validation
- **MCP Protocol Compliance**: ✅ New tools will follow existing MCP patterns (JSON Schema, structured responses)
- **Async-First Design**: ✅ All calculations will be async, non-blocking on WebSocket thread
- **Machine-Optimized Development**: ✅ Specification follows /speckit.specify workflow with testable requirements
