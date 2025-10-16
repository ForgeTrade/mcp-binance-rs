# Data Models: HTTP REST API + WebSocket

**Feature**: 003-specify-scripts-bash | **Date**: 2025-10-16

This document describes all data types used in the HTTP REST API and WebSocket streams for the Binance MCP Server.

## REST API Request Models

### Query Parameters

#### TickerPriceQuery
Query parameters for GET /api/v1/ticker/price
```rust
{
  "symbol": String  // Trading pair (e.g., "BTCUSDT")
}
```

#### Ticker24hrQuery
Query parameters for GET /api/v1/ticker/24hr
```rust
{
  "symbol": String  // Trading pair (e.g., "BTCUSDT")
}
```

#### KlinesQuery
Query parameters for GET /api/v1/klines
```rust
{
  "symbol": String,      // Trading pair (e.g., "BTCUSDT")
  "interval": String,    // Interval: "1m", "5m", "15m", "1h", "4h", "1d", "1w", "1M"
  "limit": Option<u32>   // Number of candles (default 500, max 1000)
}
```

#### DepthQuery
Query parameters for GET /api/v1/depth
```rust
{
  "symbol": String,      // Trading pair (e.g., "BTCUSDT")
  "limit": Option<u32>   // Depth levels: 5, 10, 20, 50, 100, 500, 1000, 5000
}
```

#### TradesQuery
Query parameters for GET /api/v1/trades
```rust
{
  "symbol": String,      // Trading pair (e.g., "BTCUSDT")
  "limit": Option<u32>   // Number of trades (default 500, max 1000)
}
```

#### MyTradesQuery
Query parameters for GET /api/v1/myTrades
```rust
{
  "symbol": String,      // Trading pair (e.g., "BTCUSDT")
  "limit": Option<u32>   // Number of trades (default 500, max 1000)
}
```

#### CreateOrderRequest
Request body for POST /api/v1/order
```rust
{
  "symbol": String,          // Trading pair (e.g., "BTCUSDT")
  "side": String,            // "BUY" or "SELL"
  "type": String,            // "LIMIT", "MARKET", "STOP_LOSS", etc.
  "quantity": String,        // Order quantity
  "price": Option<String>    // Required for LIMIT orders
}
```

#### CancelOrderQuery
Query parameters for DELETE /api/v1/order
```rust
{
  "symbol": String,     // Trading pair (e.g., "BTCUSDT")
  "orderId": i64        // Order ID to cancel
}
```

#### QueryOrderQuery
Query parameters for GET /api/v1/order
```rust
{
  "symbol": String,     // Trading pair (e.g., "BTCUSDT")
  "orderId": i64        // Order ID to query
}
```

#### OpenOrdersQuery
Query parameters for GET /api/v1/openOrders
```rust
{
  "symbol": Option<String>  // Optional symbol filter
}
```

#### AllOrdersQuery
Query parameters for GET /api/v1/allOrders
```rust
{
  "symbol": String,      // Trading pair (e.g., "BTCUSDT")
  "limit": Option<u32>   // Number of orders (default 500, max 1000)
}
```

## REST API Response Models

### Market Data Responses

All REST API responses are dynamic JSON (serde_json::Value) that match Binance API responses. Key fields:

#### Ticker Price Response
```json
{
  "symbol": "BTCUSDT",
  "price": "45000.50"
}
```

#### 24hr Ticker Response
```json
{
  "symbol": "BTCUSDT",
  "priceChange": "1000.00",
  "priceChangePercent": "2.27",
  "weightedAvgPrice": "44500.00",
  "prevClosePrice": "44000.00",
  "lastPrice": "45000.00",
  "lastQty": "0.01",
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

#### Kline (Candlestick) Response
Array of arrays with OHLCV data:
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
    "22250000.00"       // Taker buy quote volume
  ],
  // ... more candles
]
```

#### Order Book Depth Response
```json
{
  "lastUpdateId": 1027024,
  "bids": [
    ["44990.00", "0.5"],    // [price, quantity]
    ["44985.00", "1.2"],
    // ... more bids
  ],
  "asks": [
    ["45000.00", "0.8"],
    ["45005.00", "1.5"],
    // ... more asks
  ]
}
```

#### Recent Trades Response
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
  // ... more trades
]
```

### Order Management Responses

#### Order Response (Create/Cancel/Query)
```json
{
  "symbol": "BTCUSDT",
  "orderId": 12345678,
  "clientOrderId": "x-A6SIDXVS",
  "transactTime": 1609459200000,
  "price": "45000.00",
  "origQty": "0.01",
  "executedQty": "0.00",
  "cummulativeQuoteQty": "0.00",
  "status": "NEW",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY"
}
```

#### Open Orders Response
Array of order objects (same structure as Order Response)

### Account Information Responses

#### Account Response
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
    }
  ],
  "permissions": ["SPOT"]
}
```

#### My Trades Response
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

### User Data Stream Responses

#### Listen Key Response
```json
{
  "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
}
```

## WebSocket Stream Models

### TickerUpdate (24hr Ticker Stream)
Sent via `/ws/ticker/:symbol` every 1000ms
```rust
{
  "e": "24hrTicker",              // Event type
  "E": 1609459200000,             // Event time
  "s": "BTCUSDT",                 // Symbol
  "p": "1000.00",                 // Price change
  "P": "2.27",                    // Price change percent
  "w": "44500.00",                // Weighted average price
  "c": "45000.00",                // Last price
  "Q": "0.01",                    // Last quantity
  "o": "44000.00",                // Open price
  "h": "45500.00",                // High price
  "l": "43500.00",                // Low price
  "v": "1000.5",                  // Total traded base volume
  "q": "44500000.00"              // Total traded quote volume
}
```

**Source**: Binance `<symbol>@ticker` stream
**Frequency**: ~1000ms
**Wire Format**: JSON text frame

### DepthUpdate (Order Book Stream)
Sent via `/ws/depth/:symbol` on each order book change
```rust
{
  "e": "depthUpdate",             // Event type
  "E": 1609459200000,             // Event time
  "s": "BTCUSDT",                 // Symbol
  "U": 157,                       // First update ID
  "u": 160,                       // Final update ID
  "b": [                          // Bids to be updated
    ["44990.00", "0.5"],          // [price, quantity]
    ["44985.00", "1.2"]
  ],
  "a": [                          // Asks to be updated
    ["45000.00", "0.8"],
    ["45005.00", "1.5"]
  ]
}
```

**Source**: Binance `<symbol>@depth` stream
**Frequency**: Real-time (on each order book change)
**Wire Format**: JSON text frame

### UserDataEvent (User Data Stream)
Sent via `/ws/user` for authenticated users

#### ExecutionReport (Order Update)
```rust
{
  "e": "executionReport",          // Event type
  "E": 1609459200000,              // Event time
  "s": "BTCUSDT",                  // Symbol
  "c": "x-A6SIDXVS",               // Client order ID
  "S": "BUY",                      // Side
  "o": "LIMIT",                    // Order type
  "f": "GTC",                      // Time in force
  "q": "0.01",                     // Order quantity
  "p": "45000.00",                 // Order price
  "x": "NEW",                      // Execution type (NEW, CANCELED, TRADE, etc.)
  "X": "NEW",                      // Order status
  "r": "",                         // Order reject reason (if rejected)
  "i": 12345678,                   // Order ID
  "l": "0.00",                     // Last executed quantity
  "z": "0.00",                     // Cumulative filled quantity
  "L": "0.00",                     // Last executed price
  "n": "0.00",                     // Commission amount
  "N": null,                       // Commission asset
  "T": 1609459200000,              // Transaction time
  "t": -1,                         // Trade ID
  "w": true,                       // Is order on book?
  "m": false,                      // Is maker side?
  "O": 1609459200000,              // Order creation time
  "Z": "0.00",                     // Cumulative quote quantity
  "Y": "0.00",                     // Last quote quantity
  "Q": "0.00"                      // Quote order quantity
}
```

#### OutboundAccountPosition (Balance Update)
```rust
{
  "e": "outboundAccountPosition",  // Event type
  "E": 1609459200000,              // Event time
  "u": 1609459200000,              // Last account update time
  "B": [                           // Balances
    {
      "a": "BTC",                  // Asset
      "f": "0.00100000",           // Free
      "l": "0.00000000"            // Locked
    },
    {
      "a": "USDT",
      "f": "10000.00",
      "l": "0.00"
    }
  ]
}
```

**Source**: Binance user data stream (authenticated)
**Frequency**: Real-time on order/balance changes
**Wire Format**: JSON text frame
**Authentication**: Requires listen key from POST /api/v1/userDataStream

## Error Response Model

All HTTP error responses follow this format:
```json
{
  "error": "Error message description",
  "status": 400  // HTTP status code
}
```

Common error codes:
- **400 Bad Request**: Invalid parameters
- **401 Unauthorized**: Missing or invalid Bearer token
- **403 Forbidden**: Invalid Binance API credentials
- **429 Too Many Requests**: Rate limit exceeded
- **500 Internal Server Error**: Binance API error or server error
- **503 Service Unavailable**: Maximum WebSocket connections reached

## Data Type Conventions

### Decimal Values
All price, quantity, and volume fields are represented as **strings** to preserve precision (e.g., `"45000.50"` not `45000.5`). This matches Binance API conventions and prevents floating-point precision loss.

### Timestamps
All timestamps are **milliseconds since Unix epoch** (i64). Example: `1609459200000` = 2021-01-01 00:00:00 UTC

### Symbol Format
Trading pair symbols use uppercase concatenation: `"BTCUSDT"`, `"ETHBTC"`, etc.

### Intervals
Kline intervals: `"1m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"4h"`, `"1d"`, `"1w"`, `"1M"`

## Internal Rust Types

The server uses these internal types (from `src/binance/websocket.rs`):

```rust
// Ticker update
pub struct TickerUpdate {
    pub event_type: String,
    pub event_time: i64,
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub last_price: String,
    pub last_quantity: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
}

// Depth update
pub struct DepthUpdate {
    pub event_type: String,
    pub event_time: i64,
    pub symbol: String,
    pub first_update_id: i64,
    pub final_update_id: i64,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

// User data event (enum)
pub enum UserDataEvent {
    ExecutionReport(Box<ExecutionReport>),
    OutboundAccountPosition(OutboundAccountPosition),
}

// Execution report
pub struct ExecutionReport {
    pub event_time: i64,
    pub symbol: String,
    pub client_order_id: String,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub quantity: String,
    pub price: String,
    pub execution_type: String,
    pub order_status: String,
    pub order_reject_reason: String,
    pub order_id: i64,
    pub last_executed_quantity: String,
    pub cumulative_filled_quantity: String,
    pub last_executed_price: String,
    pub commission_amount: String,
    pub commission_asset: Option<String>,
    pub transaction_time: i64,
    pub trade_id: i64,
    pub is_order_on_book: bool,
    pub is_maker_side: bool,
    pub order_creation_time: i64,
    pub cumulative_quote_quantity: String,
    pub last_quote_quantity: String,
    pub quote_order_quantity: String,
}

// Account position update
pub struct OutboundAccountPosition {
    pub event_time: i64,
    pub last_update_time: i64,
    pub balances: Vec<BalanceUpdate>,
}

pub struct BalanceUpdate {
    pub asset: String,
    pub free: String,
    pub locked: String,
}
```

## Validation Rules

### Symbol Validation
- Must match Binance trading pairs (validated by Binance API)
- Case-insensitive (converted to uppercase internally)

### Limit Validation
- Must be ≤ 1000 for most endpoints
- Depth limit must be one of: 5, 10, 20, 50, 100, 500, 1000, 5000

### Order Side Validation
- Must be `"BUY"` or `"SELL"` (case-sensitive)

### Order Type Validation
- Supported: `"LIMIT"`, `"MARKET"`, `"STOP_LOSS"`, `"STOP_LOSS_LIMIT"`, `"TAKE_PROFIT"`, `"TAKE_PROFIT_LIMIT"`, `"LIMIT_MAKER"`
- LIMIT orders require `price` parameter

### Interval Validation
- Must be one of: `"1m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"4h"`, `"1d"`, `"1w"`, `"1M"`

---

**References**:
- Binance Spot API: https://binance-docs.github.io/apidocs/spot/en/
- Binance WebSocket Streams: https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams
- Internal types: `src/binance/websocket.rs`
