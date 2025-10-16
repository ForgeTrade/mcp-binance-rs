# Quickstart Guide: Binance MCP HTTP Server

**Feature**: 003-specify-scripts-bash | **Date**: 2025-10-16

Get started with the Binance MCP HTTP server in under 5 minutes. This guide covers all REST API and WebSocket functionality with working examples.

## Prerequisites

Before starting, you need:

1. **Bearer Token**: Authentication token for the MCP server
2. **Binance API Credentials**: API Key + Secret (for authenticated endpoints)
3. **Tools**: `curl` and `wscat` (or `websocat`)

Install wscat:
```bash
npm install -g wscat
```

## Configuration

Set up environment variables:

```bash
# MCP server authentication
export HTTP_BEARER_TOKEN="test_token_123"

# Binance API credentials (for trading)
export BINANCE_API_KEY="your_binance_api_key"
export BINANCE_API_SECRET="your_binance_secret_key"

# Server configuration (optional)
export HTTP_HOST="0.0.0.0"
export HTTP_PORT="3000"
export HTTP_RATE_LIMIT="100"  # requests per minute
export MAX_WS_CONNECTIONS="50"
```

Start the server:
```bash
cargo run --release --features http,websocket
```

Server runs at `http://localhost:3000`

## 📖 Table of Contents

- [User Story 1: Market Data via REST API](#user-story-1-market-data-via-rest-api)
- [User Story 2: Order Management via REST API](#user-story-2-order-management-via-rest-api)
- [User Story 3: Account Balance & Positions](#user-story-3-account-balance--positions)
- [User Story 4: Real-time Prices via WebSocket](#user-story-4-real-time-prices-via-websocket)
- [User Story 5: Real-time Order Book via WebSocket](#user-story-5-real-time-order-book-via-websocket)
- [User Story 6: User Data Stream via WebSocket](#user-story-6-user-data-stream-via-websocket)
- [Error Handling](#error-handling)
- [Advanced Topics](#advanced-topics)

---

## User Story 1: Market Data via REST API

### Scenario 1.1: Get Current Price

Get the latest price for BTCUSDT:

```bash
curl -X GET "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "price": "45000.50"
}
```

**Success Criteria**: ✅ Response received in < 1 second

### Scenario 1.2: Get 24hr Statistics

Get 24-hour trading statistics:

```bash
curl -X GET "http://localhost:3000/api/v1/ticker/24hr?symbol=BTCUSDT" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "priceChange": "1000.00",
  "priceChangePercent": "2.27",
  "weightedAvgPrice": "44500.00",
  "prevClosePrice": "44000.00",
  "lastPrice": "45000.00",
  "lastQty": "0.01",
  "bidPrice": "44999.00",
  "askPrice": "45001.00",
  "openPrice": "44000.00",
  "highPrice": "45500.00",
  "lowPrice": "43500.00",
  "volume": "1000.5",
  "quoteVolume": "44500000.00",
  "openTime": 1609459200000,
  "closeTime": 1609545600000,
  "count": 50000
}
```

### Scenario 1.3: Get Historical Klines (Candlesticks)

Get 100 hourly candles:

```bash
curl -X GET "http://localhost:3000/api/v1/klines?symbol=BTCUSDT&interval=1h&limit=100" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
[
  [
    1609459200000,      // Open time
    "44000.00",         // Open
    "45000.00",         // High
    "43500.00",         // Low
    "44500.00",         // Close
    "1000.5",           // Volume
    1609545599999,      // Close time
    "44500000.00",      // Quote asset volume
    50000,              // Number of trades
    "500.25",           // Taker buy base volume
    "22250000.00",      // Taker buy quote volume
    "0"                 // Unused field
  ],
  // ... 99 more candles
]
```

**Available Intervals**: `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `1d`, `1w`, `1M`

### Scenario 1.4: Get Order Book Depth

Get top 10 bid/ask levels:

```bash
curl -X GET "http://localhost:3000/api/v1/depth?symbol=BTCUSDT&limit=10" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
{
  "lastUpdateId": 1027024,
  "bids": [
    ["44990.00", "0.5"],
    ["44985.00", "1.2"],
    ["44980.00", "0.8"]
  ],
  "asks": [
    ["45000.00", "0.8"],
    ["45005.00", "1.5"],
    ["45010.00", "2.0"]
  ]
}
```

**Valid Limits**: `5`, `10`, `20`, `50`, `100`, `500`, `1000`, `5000`

### Scenario 1.5: Get Recent Trades

Get last 100 trades:

```bash
curl -X GET "http://localhost:3000/api/v1/trades?symbol=BTCUSDT&limit=100" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
[
  {
    "id": 28457,
    "price": "45000.00",
    "qty": "0.01",
    "quoteQty": "450.00",
    "time": 1609459200000,
    "isBuyerMaker": true,
    "isBestMatch": true
  },
  // ... 99 more trades
]
```

### Scenario 1.6: Error - Invalid Symbol

```bash
curl -X GET "http://localhost:3000/api/v1/ticker/price?symbol=INVALID" \
  -H "Authorization: Bearer test_token_123"
```

**Response** (HTTP 400):
```json
{
  "error": "Invalid symbol",
  "status": 400
}
```

### Scenario 1.7: Error - Missing Authorization

```bash
curl -X GET "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT"
```

**Response** (HTTP 401):
```json
{
  "error": "Missing Authorization header",
  "status": 401
}
```

---

## User Story 2: Order Management via REST API

**⚠️ Important**: Order management requires valid Binance API credentials.

### Scenario 2.1: Create Limit Order

Place a limit buy order:

```bash
curl -X POST "http://localhost:3000/api/v1/order" \
  -H "Authorization: Bearer test_token_123" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "LIMIT",
    "quantity": "0.001",
    "price": "44000.00",
    "timeInForce": "GTC"
  }'
```

**Response** (HTTP 200):
```json
{
  "symbol": "BTCUSDT",
  "orderId": 12345678,
  "clientOrderId": "x-A6SIDXVS",
  "transactTime": 1609459200000,
  "price": "44000.00",
  "origQty": "0.001",
  "executedQty": "0.000",
  "cummulativeQuoteQty": "0.00",
  "status": "NEW",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY"
}
```

**Success Criteria**: ✅ Order created and returns order ID with status "NEW"

### Scenario 2.2: Create Market Order

Place a market buy order:

```bash
curl -X POST "http://localhost:3000/api/v1/order" \
  -H "Authorization: Bearer test_token_123" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "MARKET",
    "quantity": "0.001"
  }'
```

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "orderId": 12345679,
  "status": "FILLED",
  "executedQty": "0.001",
  "cummulativeQuoteQty": "45.00",
  "fills": [
    {
      "price": "45000.00",
      "qty": "0.001",
      "commission": "0.00000100",
      "commissionAsset": "BTC"
    }
  ]
}
```

### Scenario 2.3: Query Order Status

Check order status:

```bash
curl -X GET "http://localhost:3000/api/v1/order?symbol=BTCUSDT&orderId=12345678" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "orderId": 12345678,
  "status": "NEW",
  "price": "44000.00",
  "origQty": "0.001",
  "executedQty": "0.000",
  "time": 1609459200000,
  "updateTime": 1609459200000
}
```

**Success Criteria**: ✅ Returns current order status and details

### Scenario 2.4: Cancel Order

Cancel an open order:

```bash
curl -X DELETE "http://localhost:3000/api/v1/order?symbol=BTCUSDT&orderId=12345678" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "orderId": 12345678,
  "status": "CANCELED",
  "origClientOrderId": "x-A6SIDXVS"
}
```

**Success Criteria**: ✅ Order canceled and returns status "CANCELED"

### Scenario 2.5: List Open Orders

Get all open orders for a symbol:

```bash
curl -X GET "http://localhost:3000/api/v1/openOrders?symbol=BTCUSDT" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
[
  {
    "symbol": "BTCUSDT",
    "orderId": 12345678,
    "status": "NEW",
    "price": "44000.00",
    "origQty": "0.001"
  },
  {
    "symbol": "BTCUSDT",
    "orderId": 12345680,
    "status": "PARTIALLY_FILLED",
    "price": "44500.00",
    "origQty": "0.002",
    "executedQty": "0.001"
  }
]
```

### Scenario 2.6: Get All Orders (History)

Get order history:

```bash
curl -X GET "http://localhost:3000/api/v1/allOrders?symbol=BTCUSDT&limit=100" \
  -H "Authorization: Bearer test_token_123"
```

**Response**: Array of all orders (up to limit)

### Scenario 2.7: Error - Insufficient Balance

```bash
curl -X POST "http://localhost:3000/api/v1/order" \
  -H "Authorization: Bearer test_token_123" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "LIMIT",
    "quantity": "1000.0",
    "price": "44000.00"
  }'
```

**Response** (HTTP 400):
```json
{
  "error": "Account has insufficient balance for requested action",
  "status": 400
}
```

---

## User Story 3: Account Balance & Positions

### Scenario 3.1: Get Account Information

Get full account details with balances:

```bash
curl -X GET "http://localhost:3000/api/v1/account" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
{
  "makerCommission": 10,
  "takerCommission": 10,
  "buyerCommission": 0,
  "sellerCommission": 0,
  "canTrade": true,
  "canWithdraw": true,
  "canDeposit": true,
  "updateTime": 1609459200000,
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "0.00100000",
      "locked": "0.00000000"
    },
    {
      "asset": "USDT",
      "free": "10000.00",
      "locked": "0.00"
    },
    {
      "asset": "ETH",
      "free": "0.0",
      "locked": "0.0"
    }
  ],
  "permissions": ["SPOT"]
}
```

**Success Criteria**: ✅ Shows all balances with available and locked funds

### Scenario 3.2: Get Trade History

Get your executed trades:

```bash
curl -X GET "http://localhost:3000/api/v1/myTrades?symbol=BTCUSDT&limit=50" \
  -H "Authorization: Bearer test_token_123"
```

**Response**:
```json
[
  {
    "symbol": "BTCUSDT",
    "id": 28457,
    "orderId": 100234,
    "price": "45000.00",
    "qty": "0.001",
    "quoteQty": "45.00",
    "commission": "0.045",
    "commissionAsset": "USDT",
    "time": 1609459200000,
    "isBuyer": true,
    "isMaker": false,
    "isBestMatch": true
  },
  // ... more trades
]
```

### Scenario 3.3: Monitor Balance After Trade

Check balance update after trade execution:

```bash
# Before trade
curl -X GET "http://localhost:3000/api/v1/account" \
  -H "Authorization: Bearer test_token_123" | jq '.balances[] | select(.asset=="USDT")'

# Execute trade
curl -X POST "http://localhost:3000/api/v1/order" \
  -H "Authorization: Bearer test_token_123" \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTCUSDT", "side": "BUY", "type": "MARKET", "quantity": "0.001"}'

# After trade
curl -X GET "http://localhost:3000/api/v1/account" \
  -H "Authorization: Bearer test_token_123" | jq '.balances[] | select(.asset=="USDT")'
```

**Success Criteria**: ✅ Balance reflects trade execution

---

## User Story 4: Real-time Prices via WebSocket

### Scenario 4.1: Connect to Ticker Stream

Subscribe to real-time BTCUSDT price updates:

```bash
wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123"
```

**Messages** (received every ~1 second):
```json
{
  "e": "24hrTicker",
  "E": 1609459200000,
  "s": "BTCUSDT",
  "p": "1000.00",
  "P": "2.27",
  "w": "44500.00",
  "c": "45000.00",
  "Q": "0.01",
  "o": "44000.00",
  "h": "45500.00",
  "l": "43500.00",
  "v": "1000.5",
  "q": "44500000.00"
}
```

**Success Criteria**: ✅ Receives price updates with < 500ms latency

### Scenario 4.2: Monitor Price with jq

Pretty-print only the current price:

```bash
wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '.c + " @ " + (.E/1000|todate)'
```

**Output**:
```
45000.00 @ 2025-01-01T00:00:00Z
45001.50 @ 2025-01-01T00:00:01Z
45002.00 @ 2025-01-01T00:00:02Z
```

### Scenario 4.3: Alert on Price Change

Script to alert when price changes by >1%:

```bash
#!/bin/bash
LAST_PRICE=0

wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '.c' \
  | while read PRICE; do
    if [ "$LAST_PRICE" != "0" ]; then
      CHANGE=$(echo "scale=2; ($PRICE - $LAST_PRICE) / $LAST_PRICE * 100" | bc)
      if (( $(echo "$CHANGE > 1 || $CHANGE < -1" | bc -l) )); then
        echo "🚨 ALERT: Price changed by ${CHANGE}%"
      fi
    fi
    LAST_PRICE=$PRICE
  done
```

### Scenario 4.4: Reconnection on Disconnect

WebSocket automatically reconnects if connection drops. Test:

```bash
# Connect
wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123"

# Kill connection (Ctrl+C)
# Reconnect - data resumes
```

**Success Criteria**: ✅ Can reconnect without data loss

---

## User Story 5: Real-time Order Book via WebSocket

### Scenario 5.1: Connect to Depth Stream

Subscribe to real-time BTCUSDT order book:

```bash
wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token_123"
```

**Messages** (on each order book change):
```json
{
  "e": "depthUpdate",
  "E": 1609459200000,
  "s": "BTCUSDT",
  "U": 157,
  "u": 160,
  "b": [
    ["44990.00", "0.5"],
    ["44985.00", "1.2"]
  ],
  "a": [
    ["45000.00", "0.8"],
    ["45005.00", "1.5"]
  ]
}
```

**Success Criteria**: ✅ Receives order book updates in real-time

### Scenario 5.2: Monitor Best Bid/Ask

Extract only top bid/ask:

```bash
wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '.b[0][0] + " / " + .a[0][0]'
```

**Output**:
```
44990.00 / 45000.00
44991.00 / 45000.00
44991.00 / 44999.00
```

### Scenario 5.3: Calculate Spread

Monitor bid-ask spread:

```bash
wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '(.a[0][0] | tonumber) - (.b[0][0] | tonumber)'
```

**Output**:
```
10.00
9.00
8.00
```

### Scenario 5.4: High Volatility Test

During high volatility, updates come rapidly:

```bash
wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '.E'
```

**Success Criteria**: ✅ All updates delivered with <200ms latency, no missed updates

---

## User Story 6: User Data Stream via WebSocket

### Scenario 6.1: Connect to User Stream

Subscribe to your order and balance updates:

```bash
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123"
```

**Server automatically**:
- Creates listen key on connect
- Renews listen key every 30 minutes
- Closes listen key on disconnect

**Messages** (on order/balance changes):

**Execution Report** (order update):
```json
{
  "e": "executionReport",
  "E": 1609459200000,
  "s": "BTCUSDT",
  "c": "x-A6SIDXVS",
  "S": "BUY",
  "o": "LIMIT",
  "q": "0.01",
  "p": "45000.00",
  "x": "TRADE",
  "X": "FILLED",
  "i": 12345678,
  "z": "0.01"
}
```

**Account Position** (balance update):
```json
{
  "e": "outboundAccountPosition",
  "E": 1609459200000,
  "u": 1609459200000,
  "B": [
    {
      "a": "BTC",
      "f": "0.01100000",
      "l": "0.00000000"
    },
    {
      "a": "USDT",
      "f": "9550.00",
      "l": "0.00"
    }
  ]
}
```

**Success Criteria**: ✅ Instant notifications on order/balance changes

### Scenario 6.2: Monitor Order Execution

Filter only execution reports:

```bash
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r 'select(.e=="executionReport") | .X + " order " + (.i|tostring) + " @ " + .p'
```

**Output**:
```
NEW order 12345678 @ 45000.00
PARTIALLY_FILLED order 12345678 @ 45000.00
FILLED order 12345678 @ 45000.00
```

### Scenario 6.3: Monitor Balance Changes

Filter only balance updates:

```bash
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r 'select(.e=="outboundAccountPosition") | .B[] | select(.a=="USDT") | "USDT: " + .f'
```

**Output**:
```
USDT: 10000.00
USDT: 9955.00
USDT: 9550.00
```

### Scenario 6.4: Combined Trading Flow

Complete trading flow with real-time updates:

```bash
# Terminal 1: Monitor user stream
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123"

# Terminal 2: Place order
curl -X POST "http://localhost:3000/api/v1/order" \
  -H "Authorization: Bearer test_token_123" \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTCUSDT", "side": "BUY", "type": "MARKET", "quantity": "0.001"}'

# Terminal 1: See execution report + balance update
```

**Success Criteria**: ✅ Real-time notifications without polling

### Scenario 6.5: Automatic Listen Key Renewal

Listen key is automatically renewed every 30 minutes:

```bash
# Connect and leave running for >30 minutes
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123"

# Connection remains stable, no manual renewal needed
```

**Success Criteria**: ✅ Connection stays alive for hours without intervention

---

## Error Handling

### Authentication Errors

**Missing Bearer Token**:
```bash
curl -X GET "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT"
```

**Response** (HTTP 401):
```json
{
  "error": "Missing Authorization header",
  "status": 401
}
```

**Invalid Bearer Token**:
```bash
curl -X GET "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT" \
  -H "Authorization: Bearer wrong_token"
```

**Response** (HTTP 401):
```json
{
  "error": "Invalid or expired token",
  "status": 401
}
```

### Rate Limiting

**Too Many Requests**:
```bash
for i in {1..200}; do
  curl -X GET "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT" \
    -H "Authorization: Bearer test_token_123"
done
```

**Response** (HTTP 429):
```json
{
  "error": "Rate limit exceeded",
  "status": 429
}
```

**Headers**:
```
HTTP/1.1 429 Too Many Requests
Retry-After: 60
```

### Connection Limits

**WebSocket Connection Limit**:

```bash
# Open 51 WebSocket connections
for i in {1..51}; do
  wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
    -H "Authorization: Bearer test_token_123" &
done
```

**51st Connection** (HTTP 503):
```json
{
  "error": "Maximum WebSocket connections reached",
  "status": 503
}
```

**Headers**:
```
HTTP/1.1 503 Service Unavailable
Retry-After: 60
```

### Binance API Errors

**Invalid Binance Credentials**:

```bash
export BINANCE_API_KEY="invalid_key"

curl -X GET "http://localhost:3000/api/v1/account" \
  -H "Authorization: Bearer test_token_123"
```

**Response** (HTTP 403):
```json
{
  "error": "Invalid API key",
  "status": 403
}
```

**Binance Timeout**:

If Binance doesn't respond within 10 seconds:

**Response** (HTTP 504):
```json
{
  "error": "Gateway timeout - Binance API not responding",
  "status": 504
}
```

---

## Advanced Topics

### Health Check

Check server health:

```bash
curl -X GET "http://localhost:3000/health"
```

**Response**:
```json
{
  "status": "ok",
  "timestamp": 1609459200000,
  "uptime_seconds": 3600
}
```

### Request Tracing

All responses include tracing headers:

```bash
curl -i -X GET "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT" \
  -H "Authorization: Bearer test_token_123"
```

**Headers**:
```
X-Request-Id: 550e8400-e29b-41d4-a716-446655440000
X-Response-Time: 123
```

### CORS Configuration

For browser-based applications:

```bash
curl -i -X OPTIONS "http://localhost:3000/api/v1/ticker/price" \
  -H "Origin: http://example.com" \
  -H "Access-Control-Request-Method: GET"
```

**Response**:
```
HTTP/1.1 200 OK
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS
Access-Control-Allow-Headers: Authorization, Content-Type
```

### Logging

Server logs all requests (sanitized):

```
[INFO] HTTP GET /api/v1/ticker/price?symbol=BTCUSDT - 200 - 123ms
[INFO] WebSocket connected: /ws/ticker/btcusdt - client_id=abc123
[ERROR] Binance API error: rate limit exceeded - retrying in 60s
[INFO] WebSocket disconnected: /ws/ticker/btcusdt - duration=3600s
```

### Performance Tips

1. **Reuse connections**: Use keep-alive for REST API
2. **WebSocket for real-time**: Prefer WebSocket over polling
3. **Batch requests**: Get multiple symbols in one klines request (comma-separated)
4. **Cache ticker data**: Only query when needed, use WebSocket for monitoring
5. **Handle rate limits**: Implement exponential backoff on 429 errors

### Integration Examples

**Python**:
```python
import requests
import websocket

# REST API
headers = {"Authorization": "Bearer test_token_123"}
response = requests.get(
    "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT",
    headers=headers
)
print(response.json())

# WebSocket
ws = websocket.WebSocketApp(
    "ws://localhost:3000/ws/ticker/btcusdt",
    header={"Authorization": "Bearer test_token_123"},
    on_message=lambda ws, msg: print(msg)
)
ws.run_forever()
```

**JavaScript**:
```javascript
// REST API
const response = await fetch(
  "http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT",
  {
    headers: { "Authorization": "Bearer test_token_123" }
  }
);
const data = await response.json();
console.log(data);

// WebSocket
const ws = new WebSocket("ws://localhost:3000/ws/ticker/btcusdt");
ws.onopen = () => {
  // Note: Authorization header must be set during connection upgrade
  // Use a library like ws that supports custom headers
};
ws.onmessage = (event) => console.log(JSON.parse(event.data));
```

---

## Troubleshooting

### Issue: "Connection refused"

**Cause**: Server not running or wrong port

**Solution**:
```bash
# Check server is running
lsof -i :3000

# Start server
cargo run --release --features http,websocket
```

### Issue: "Invalid or expired token"

**Cause**: Bearer token not configured

**Solution**:
```bash
# Set token in environment
export HTTP_BEARER_TOKEN="test_token_123"

# Restart server
```

### Issue: "Maximum WebSocket connections reached"

**Cause**: Too many open connections (>50)

**Solution**:
```bash
# Close unused connections
# Or increase limit
export MAX_WS_CONNECTIONS="100"
```

### Issue: WebSocket messages delayed

**Cause**: Binance upstream connection slow

**Solution**:
- Check network latency to Binance API
- Ensure server has stable internet connection
- Monitor server logs for reconnection attempts

---

## Next Steps

- 📖 **OpenAPI Spec**: See `contracts/openapi.yaml` for complete API reference
- 📡 **WebSocket Protocol**: See `contracts/websocket-streams.md` for protocol details
- 🔧 **Configuration**: See main README for environment variables
- 🧪 **Integration Tests**: Run `cargo test --features http,websocket`

---

**Need Help?**

- Check server logs for detailed error messages
- Ensure Binance API credentials have correct permissions
- Verify network connectivity to Binance API
- Review OpenAPI spec for parameter requirements

**API Documentation**:
- Binance Spot API: https://binance-docs.github.io/apidocs/spot/en/
- WebSocket Streams: https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams
