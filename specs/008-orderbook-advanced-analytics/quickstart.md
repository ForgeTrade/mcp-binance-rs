# Feature 008: Advanced Order Book Analytics - Quick Start Guide

## Prerequisites

- **Rust**: 1.90+ with Edition 2024
- **Cargo Features**: Compile with `--features orderbook_analytics`
- **Binance API**: Production API key/secret (no testnet support for real-time streams)
- **MCP Client**: Claude Desktop, Zed, or any MCP-compatible client

## Installation

1. **Build with analytics feature**:
```bash
cargo build --release --features orderbook_analytics
```

2. **Configure MCP server** (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "binance": {
      "command": "/path/to/mcp-binance-rs/target/release/mcp-binance-rs",
      "env": {
        "BINANCE_API_KEY": "your_api_key",
        "BINANCE_API_SECRET": "your_api_secret"
      }
    }
  }
}
```

3. **Verify installation**:
```bash
# List available tools - should include 5 new analytics tools
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run --features orderbook_analytics
```

## Usage Scenarios

### Scenario 1: Real-Time Order Flow Analysis

**Use Case**: Monitor buying/selling pressure on BTCUSDT over 60-second window

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_order_flow",
    "arguments": {
      "symbol": "BTCUSDT",
      "time_window_secs": 60
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"symbol\":\"BTCUSDT\",\"time_window_start\":\"2025-01-18T10:00:00Z\",\"time_window_end\":\"2025-01-18T10:01:00Z\",\"window_duration_secs\":60,\"bid_flow_rate\":12.5,\"ask_flow_rate\":8.3,\"net_flow\":4.2,\"flow_direction\":\"MODERATE_BUY\",\"cumulative_delta\":252.8}"
    }]
  }
}
```

**Interpretation**: Net flow +4.2 (12.5 bid orders/sec - 8.3 ask orders/sec) indicates moderate buying pressure.

---

### Scenario 2: Volume Profile Analysis

**Use Case**: Identify Point of Control (POC) and Value Area for ETHUSDT over 24 hours

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_volume_profile",
    "arguments": {
      "symbol": "ETHUSDT",
      "time_period_hours": 24
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"symbol\":\"ETHUSDT\",\"time_period_start\":\"2025-01-17T10:00:00Z\",\"time_period_end\":\"2025-01-18T10:00:00Z\",\"price_range_low\":\"3200.50\",\"price_range_high\":\"3450.80\",\"bin_size\":\"2.50\",\"bin_count\":100,\"histogram\":[{\"price_level\":\"3325.00\",\"volume\":\"128450.25\",\"trade_count\":4582}],\"total_volume\":\"5420150.50\",\"point_of_control\":\"3325.00\",\"value_area_high\":\"3375.00\",\"value_area_low\":\"3280.00\"}"
    }]
  }
}
```

**Interpretation**: POC at $3325 shows highest volume concentration. 70% of volume traded between $3280-$3375 (Value Area).

---

### Scenario 3: Anomaly Detection

**Use Case**: Detect quote stuffing, iceberg orders, or flash crash precursors

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "detect_market_anomalies",
    "arguments": {
      "symbol": "BTCUSDT",
      "lookback_seconds": 60
    }
  }
}
```

**Response (No Anomalies)**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"symbol\":\"BTCUSDT\",\"anomalies\":[]}"
    }]
  }
}
```

**Response (Quote Stuffing Detected)**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"symbol\":\"BTCUSDT\",\"anomalies\":[{\"anomaly_id\":\"550e8400-e29b-41d4-a716-446655440000\",\"anomaly_type\":\"QuoteStuffing\",\"detection_timestamp\":\"2025-01-18T10:01:30Z\",\"confidence_score\":0.92,\"severity\":\"High\",\"recommended_action\":\"Monitor for order cancellations. Possible HFT manipulation.\",\"metadata\":{\"update_rate\":650,\"fill_rate\":0.08}}]}"
    }]
  }
}
```

**Interpretation**: 650 updates/sec with 8% fill rate exceeds quote stuffing threshold (>500 updates/sec, <10% fills).

---

### Scenario 4: Liquidity Vacuum Detection

**Use Case**: Identify price ranges with thin orderbook depth

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "get_liquidity_vacuums",
    "arguments": {
      "symbol": "ETHUSDT",
      "time_period_hours": 24
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"symbol\":\"ETHUSDT\",\"vacuums\":[{\"vacuum_id\":\"7c9e6679-7425-40de-944b-e07fc1f90ae7\",\"price_range_low\":\"3350.00\",\"price_range_high\":\"3365.00\",\"volume_deficit_pct\":82.5,\"median_volume\":\"45000.00\",\"actual_volume\":\"7875.00\",\"expected_impact\":\"FastMovement\",\"detection_timestamp\":\"2025-01-18T10:00:00Z\"}]}"
    }]
  }
}
```

**Interpretation**: Price range $3350-$3365 has 82.5% less volume than median. Price could move rapidly through this vacuum.

---

### Scenario 5: Microstructure Health Score

**Use Case**: Get composite health metric (0-100) combining spread stability, liquidity depth, flow balance, and update rate

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "get_microstructure_health",
    "arguments": {
      "symbol": "BTCUSDT"
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"symbol\":\"BTCUSDT\",\"health_score\":78.5,\"components\":{\"spread_stability\":85.2,\"liquidity_depth\":72.8,\"order_flow_balance\":81.0,\"update_rate_normality\":75.0},\"timestamp\":\"2025-01-18T10:00:00Z\",\"prediction\":{\"expected_volatility_next_5min\":0.0042,\"confidence\":0.75}}"
    }]
  }
}
```

**Interpretation**: Health score 78.5/100 indicates healthy market. Predicted 5-min volatility: 0.42% with 75% confidence.

---

## Troubleshooting

### Issue: "Feature 'orderbook_analytics' not enabled"

**Solution**: Rebuild with feature flag:
```bash
cargo clean
cargo build --release --features orderbook_analytics
```

### Issue: "WebSocket connection failed"

**Symptoms**: Order flow or volume profile returns stale data

**Solution**: Verify production API credentials (testnet not supported):
```bash
# Test API connectivity
curl -H "X-MBX-APIKEY: your_api_key" https://api.binance.com/api/v3/account
```

### Issue: Query latency >200ms

**Symptoms**: Slow historical data retrieval

**Possible Causes**:
1. RocksDB compaction running → Wait for background compaction to complete
2. 7+ days of snapshots accumulated → Check retention cleanup task (runs hourly)
3. Large time range query → Reduce query window or use pagination

**Debug**:
```bash
# Check RocksDB size
du -sh ~/.cache/mcp-binance-rs/rocksdb

# Monitor query performance
RUST_LOG=mcp_binance_rs::orderbook::analytics=debug cargo run --features orderbook_analytics
```

### Issue: "Invalid symbol pattern"

**Symptoms**: "symbol must match ^[A-Z]{4,12}$"

**Solution**: Use uppercase symbols with 4-12 characters (e.g., "BTCUSDT", not "btcusdt" or "BTC-USDT")

---

## Success Checklist

Verify Feature 008 is working correctly:

- [ ] **AC-001**: `get_order_flow` returns bid/ask flow rates with FlowDirection enum
- [ ] **AC-002**: `get_volume_profile` generates histogram with POC, VAH, VAL (adaptive bins)
- [ ] **AC-003**: `detect_market_anomalies` detects quote stuffing (>500 updates/sec, <10% fills)
- [ ] **AC-004**: `detect_market_anomalies` detects iceberg orders (>5x median refill rate)
- [ ] **AC-005**: `detect_market_anomalies` detects flash crash risk (>80% depth loss)
- [ ] **AC-006**: `get_liquidity_vacuums` identifies price ranges with <20% median volume
- [ ] **AC-007**: `get_microstructure_health` returns 0-100 composite score with 4 components
- [ ] **AC-008**: Health score correlates >0.7 with 5-min volatility (validate over time)
- [ ] **NFR-001**: Historical queries complete in <200ms (test with 24h time range)
- [ ] **NFR-002**: 1-second snapshot frequency maintained during high volatility
- [ ] **NFR-003**: 7-day retention enforced (check no snapshots older than 7 days)

**Validation Command**:
```bash
# Run all analytics tools sequentially
for tool in get_order_flow get_volume_profile detect_market_anomalies get_liquidity_vacuums get_microstructure_health; do
  echo "Testing $tool..."
  echo "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"$tool\",\"arguments\":{\"symbol\":\"BTCUSDT\"}}}" | cargo run --quiet --features orderbook_analytics
done
```

---

## Additional Resources

- **Specification**: `specs/008-orderbook-advanced-analytics/spec.md`
- **Data Model**: `specs/008-orderbook-advanced-analytics/data-model.md`
- **Research**: `specs/008-orderbook-advanced-analytics/research.md`
- **API Contracts**: `specs/008-orderbook-advanced-analytics/contracts/*.json`
- **Binance API Docs**: https://binance-docs.github.io/apidocs/spot/en/#websocket-streams
