# Quickstart: Order Book Depth Tools

**Feature**: 007-orderbook-depth-tools
**Date**: 2025-10-17
**Status**: Implementation Guide

## Overview

This guide demonstrates the progressive disclosure workflow for order book depth analysis: L1 metrics → L2-lite → L2-full. Each level provides more detail at the cost of additional tokens.

---

## Prerequisites

1. **MCP Server Running**:
   ```bash
   cargo run --features orderbook
   ```

2. **MCP Client** (e.g., Claude Desktop, Claude Code):
   - Configure Binance MCP server in client settings
   - No API keys required for public order book data

3. **Valid Trading Pair**:
   - Use Binance spot symbols: BTCUSDT, ETHUSDT, BNBBTC, etc.
   - Case-insensitive, will be uppercased automatically

---

## Scenario 1: Quick Spread Assessment (L1)

**Goal**: Check if current spread and liquidity favor entering a position.

**Tool**: `get_orderbook_metrics`
**Token Cost**: ~15% of L2-full
**Latency**: <200ms (after first request)

### Example Request

```json
{
  "tool": "get_orderbook_metrics",
  "arguments": {
    "symbol": "BTCUSDT"
  }
}
```

### Example Response

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
    "bids": [
      {"price": "67600.00", "qty": "5.5", "side": "Bid"}
    ],
    "asks": [
      {"price": "67750.00", "qty": "6.2", "side": "Ask"}
    ]
  },
  "slippage_estimates": {
    "buy_10k_usd": {
      "target_usd": 10000,
      "avg_price": 67662.1,
      "slippage_bps": 1.8,
      "filled_qty": 0.1478,
      "filled_usd": 10000.0
    },
    "sell_10k_usd": {
      "target_usd": 10000,
      "avg_price": 67638.0,
      "slippage_bps": 1.9,
      "filled_qty": 0.1478,
      "filled_usd": 10000.0
    }
  }
}
```

### Interpretation

- **Spread**: 0.49 bps (tight, good for trading)
- **Microprice**: 67650.31 (fair value between bid/ask)
- **Imbalance**: 1.218 (more buy pressure, 22% bid-heavy)
- **Walls**: Support at 67600.00 (5.5 BTC), Resistance at 67750.00 (6.2 BTC)
- **Slippage**: ~1.8 bps for $10k buy, ~1.9 bps for $10k sell (low slippage)

**Decision**: Tight spread, decent liquidity, low slippage → **Favorable for entry**

---

## Scenario 2: Detailed Support/Resistance Analysis (L2-lite)

**Goal**: Identify precise support/resistance zones beyond walls for technical analysis.

**Tool**: `get_orderbook_depth`
**Token Cost**: ~50% of L2-full
**Latency**: <300ms

### Example Request

```json
{
  "tool": "get_orderbook_depth",
  "arguments": {
    "symbol": "BTCUSDT",
    "levels": 20
  }
}
```

### Example Response

```json
{
  "symbol": "BTCUSDT",
  "timestamp": 1699999999123,
  "price_scale": 100,
  "qty_scale": 100000,
  "bids": [
    [6765050, 123400],   // 67650.50 @ 1.23400 BTC
    [6765000, 80000],    // 67650.00 @ 0.80000 BTC
    [6764950, 45600],    // 67649.50 @ 0.45600 BTC
    [6764900, 123000],   // 67649.00 @ 1.23000 BTC (accumulation zone)
    [6764850, 67800]     // 67648.50 @ 0.67800 BTC
  ],
  "asks": [
    [6765100, 98700],    // 67651.00 @ 0.98700 BTC
    [6765150, 40000],    // 67651.50 @ 0.40000 BTC
    [6765200, 156000],   // 67652.00 @ 1.56000 BTC (resistance zone)
    [6765250, 88900],    // 67652.50 @ 0.88900 BTC
    [6765300, 55000]     // 67653.00 @ 0.55000 BTC
  ]
}
```

### Decoding

```python
def decode_level(scaled_price, scaled_qty):
    price = scaled_price / 100.0
    qty = scaled_qty / 100000.0
    return price, qty

# Example
price, qty = decode_level(6765050, 123400)
# → price=67650.50, qty=1.23400
```

### Interpretation

- **Support Zones**: 67649.00 (1.23 BTC accumulation), 67650.50 (1.23 BTC)
- **Resistance Zones**: 67652.00 (1.56 BTC wall), 67651.00 (0.99 BTC)
- **Analysis**: Strong resistance at 67652.00, breaking this level may trigger bullish move

**Decision**: Set limit buy at 67649.00 (support), take profit at 67652.00 (resistance)

---

## Scenario 3: Deep Market Microstructure (L2-full)

**Goal**: Analyze entire order book for large order placement or algorithmic trading.

**Tool**: `get_orderbook_depth`
**Token Cost**: 100% (baseline)
**Latency**: <300ms

### Example Request

```json
{
  "tool": "get_orderbook_depth",
  "arguments": {
    "symbol": "BTCUSDT",
    "levels": 100
  }
}
```

### Example Response

```json
{
  "symbol": "BTCUSDT",
  "timestamp": 1699999999123,
  "price_scale": 100,
  "qty_scale": 100000,
  "bids": [
    // ... 100 bid levels ...
  ],
  "asks": [
    // ... 100 ask levels ...
  ]
}
```

### Use Cases

- **Large Order Placement**: Estimate impact of 100 BTC order across 100 levels
- **VWAP Calculation**: Custom slippage estimates beyond 10k/25k/50k targets
- **Algorithmic Trading**: Detect hidden liquidity, iceberg orders, market maker patterns
- **Statistical Arbitrage**: Analyze order book shape for mean reversion signals

---

## Scenario 4: Health Check Before Critical Operation

**Goal**: Verify order book data is fresh before placing large order.

**Tool**: `get_orderbook_health`
**Token Cost**: Minimal
**Latency**: <100ms

### Example Request

```json
{
  "tool": "get_orderbook_health",
  "arguments": {}
}
```

### Example Response (Healthy)

```json
{
  "status": "ok",
  "orderbook_symbols_active": 5,
  "last_update_age_ms": 150,
  "websocket_connected": true,
  "timestamp": 1699999999123,
  "reason": null
}
```

### Example Response (Degraded)

```json
{
  "status": "degraded",
  "orderbook_symbols_active": 3,
  "last_update_age_ms": 6500,
  "websocket_connected": true,
  "timestamp": 1699999999456,
  "reason": "2 WebSocket connections reconnecting, data staleness >5s"
}
```

### Interpretation

- **ok**: Data fresh (<5s), safe to trade
- **degraded**: Stale data (>5s) or partial connectivity, proceed with caution
- **error**: All connections down, wait for recovery

**Decision**: Only execute critical orders when status="ok"

---

## Progressive Disclosure Workflow

### Recommended Decision Tree

```
1. Start: get_orderbook_metrics (L1)
   ├─ Spread tight + imbalance favorable?
   │  ├─ Yes: Consider entry based on L1 metrics
   │  └─ No: Exit (unfavorable conditions)
   │
2. Need support/resistance levels?
   ├─ Yes: get_orderbook_depth (levels=20, L2-lite)
   │  ├─ Support/resistance clear?
   │  │  ├─ Yes: Set limit orders
   │  │  └─ No: Escalate to L2-full
   │  │
   │  └─ Escalate: get_orderbook_depth (levels=100, L2-full)
   │
3. Before execution: get_orderbook_health
   ├─ Status "ok"? → Execute
   └─ Status "degraded"/"error"? → Wait or abort
```

### Token Economy Example

**Scenario**: Check 3 symbols for entry opportunities

| Workflow | Tools Called | Token Usage |
|----------|-------------|-------------|
| **Always L2-full** | 3x get_orderbook_depth(levels=100) | 100% (baseline) |
| **Progressive Disclosure** | 3x get_orderbook_metrics → 1x get_orderbook_depth(levels=20) → 0x L2-full | ~35% (65% savings) |

**Insight**: 90% of decisions made from L1, only 10% escalate to L2-lite, <1% need L2-full

---

## Error Handling

### Common Errors

**SYMBOL_NOT_FOUND**:
```json
{
  "error": {
    "code": "SYMBOL_NOT_FOUND",
    "message": "Symbol not found or not supported. Verify symbol is a valid Binance spot trading pair."
  }
}
```
**Fix**: Verify symbol at https://www.binance.com/en/markets

**SYMBOL_LIMIT_REACHED**:
```json
{
  "error": {
    "code": "SYMBOL_LIMIT_REACHED",
    "message": "Maximum concurrent symbols (20) reached. Close unused symbols to free capacity."
  }
}
```
**Fix**: Call get_orderbook_metrics for different symbol less frequently, or implement symbol rotation

**RATE_LIMIT_EXCEEDED**:
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Request queue full, rate limit exceeded. Retry after delay."
  }
}
```
**Fix**: Reduce request rate, implement exponential backoff, or wait 30s for queue to drain

---

## Performance Characteristics

### Latency Targets

| Metric | Target | Notes |
|--------|--------|-------|
| L1 metrics (warm) | <200ms | After WebSocket initialized |
| L2 depth (warm) | <300ms | From local cache |
| First request (cold) | 2-3s | REST snapshot + WebSocket connect |
| Health check | <100ms | No external calls |

### Token Economy

| Response Type | Tokens (approx) | vs L2-full |
|---------------|-----------------|------------|
| L1 metrics | 300-500 | 15% |
| L2-lite (20 levels) | 1000-1500 | 50% |
| L2-full (100 levels) | 2000-3000 | 100% (baseline) |
| Health | 50-100 | 2.5% |

### Memory Footprint

- **Per Symbol**: ~6-8 KB (100 levels * 2 sides * 32 bytes per level)
- **20 Symbols Max**: ~120-160 KB total

---

## Testing Checklist

After implementation, verify:

- [ ] L1 metrics return within 200ms (warm)
- [ ] First request completes within 3s (cold start)
- [ ] Spread calculation accuracy within 0.01 bps (FR-008, SC-008)
- [ ] Microprice calculation within $0.01 for BTCUSDT (SC-009)
- [ ] Slippage estimates within 5% of actual (SC-010)
- [ ] Wall detection flags 2x median levels (SC-011)
- [ ] Compact format reduces JSON size by ≥35% (SC-003)
- [ ] Symbol limit enforcement at 20 (FR-021)
- [ ] Rate limiter prevents 418/429 errors (SC-017)
- [ ] WebSocket reconnects within 30s in 99% of cases (SC-007)

---

**Quickstart Status**: ✅ **COMPLETE** - Ready for implementation
