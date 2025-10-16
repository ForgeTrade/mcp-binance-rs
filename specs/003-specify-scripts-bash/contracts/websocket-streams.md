# WebSocket Streams Protocol

**Feature**: 003-specify-scripts-bash | **Date**: 2025-10-16

Complete WebSocket protocol documentation for real-time Binance data streams.

## Overview

The Binance MCP HTTP server provides three WebSocket streams for real-time data:

| Stream | Endpoint | Authentication | Data Type | Frequency |
|--------|----------|----------------|-----------|-----------|
| 24hr Ticker | `/ws/ticker/{symbol}` | Bearer token | Price/volume stats | ~1000ms |
| Order Book Depth | `/ws/depth/{symbol}` | Bearer token | Bid/ask updates | Real-time |
| User Data | `/ws/user` | Bearer token + auto listen key | Orders/balances | Real-time |

**Connection Limits**: Maximum 50 concurrent WebSocket connections per server instance.

## Connection Protocol

### WebSocket Upgrade Flow

1. **Client sends HTTP Upgrade request** with Bearer token:
```http
GET /ws/ticker/btcusdt HTTP/1.1
Host: localhost:3000
Upgrade: websocket
Connection: Upgrade
Authorization: Bearer test_token_123
Sec-WebSocket-Version: 13
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
```

2. **Server validates authentication**:
   - Extracts Bearer token from `Authorization` header
   - Validates token against configured tokens (SHA-256 hash match)
   - Returns `401 Unauthorized` if missing/invalid

3. **Server checks connection limits**:
   - Tracks active WebSocket connections
   - Returns `503 Service Unavailable` with `Retry-After: 60` if limit reached
   - Limit: 50 concurrent connections

4. **Server responds with upgrade confirmation**:
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

5. **Connection established** - Server begins streaming data

### Authentication

All WebSocket endpoints require Bearer token authentication:

```bash
# Using wscat (recommended for testing)
wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123"

# Using websocat
websocat -H "Authorization: Bearer test_token_123" \
  ws://localhost:3000/ws/ticker/btcusdt
```

**Token Configuration**: Tokens are configured via `HTTP_BEARER_TOKEN` environment variable (comma-separated for multiple tokens).

## Message Format

### Wire Format

All messages are **JSON text frames** (UTF-8 encoded). Binary frames are not used.

```
WebSocket Text Frame
├── Opcode: 0x1 (text)
├── Payload Length: variable
└── Payload Data: JSON string
```

### Message Structure

All stream messages follow this pattern:

```json
{
  "e": "eventType",        // Event type identifier
  "E": 1609459200000,      // Event time (milliseconds since epoch)
  "s": "BTCUSDT",          // Symbol (uppercase)
  // ... event-specific fields
}
```

## Stream 1: 24hr Ticker (`/ws/ticker/{symbol}`)

### Connection

```bash
wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123"
```

### Message Flow

```
Client                          Server
  |                               |
  |-- WebSocket Upgrade -------->|
  |<-- 101 Switching Protocols --|
  |                               |
  |<----- TickerUpdate (1s) -----|
  |<----- TickerUpdate (1s) -----|
  |<----- TickerUpdate (1s) -----|
  |                               |
  |-- Close Frame -------------->|
  |<-- Close Frame --------------|
```

**Frequency**: ~1000ms (1 second) per message
**Upstream Source**: Binance `<symbol>@ticker` WebSocket stream

### Message Example

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

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `e` | string | Event type: `"24hrTicker"` |
| `E` | i64 | Event time (milliseconds) |
| `s` | string | Symbol (e.g., `"BTCUSDT"`) |
| `p` | string | Price change (24hr) |
| `P` | string | Price change percent (24hr) |
| `w` | string | Weighted average price (24hr) |
| `c` | string | Last price (current) |
| `Q` | string | Last quantity |
| `o` | string | Open price (24hr ago) |
| `h` | string | High price (24hr) |
| `l` | string | Low price (24hr) |
| `v` | string | Total traded base volume (24hr) |
| `q` | string | Total traded quote volume (24hr) |

**Note**: All price/quantity fields are **strings** to preserve decimal precision.

## Stream 2: Order Book Depth (`/ws/depth/{symbol}`)

### Connection

```bash
wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token_123"
```

### Message Flow

```
Client                          Server
  |                               |
  |-- WebSocket Upgrade -------->|
  |<-- 101 Switching Protocols --|
  |                               |
  |<----- DepthUpdate (real-time) --|
  |<----- DepthUpdate (real-time) --|
  |<----- DepthUpdate (real-time) --|
  |                               |
  |-- Close Frame -------------->|
  |<-- Close Frame --------------|
```

**Frequency**: Real-time on every order book change
**Upstream Source**: Binance `<symbol>@depth` WebSocket stream

### Message Example

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

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `e` | string | Event type: `"depthUpdate"` |
| `E` | i64 | Event time (milliseconds) |
| `s` | string | Symbol (e.g., `"BTCUSDT"`) |
| `U` | i64 | First update ID in this event |
| `u` | i64 | Final update ID in this event |
| `b` | array | Bids to update `[[price, quantity], ...]` |
| `a` | array | Asks to update `[[price, quantity], ...]` |

### Order Book Update Logic

```
For each bid/ask in the update:
  if quantity == "0.00000000":
    remove_price_level(price)
  else:
    update_price_level(price, quantity)
```

**Note**: Quantity of `"0.00000000"` means remove that price level from the book.

## Stream 3: User Data (`/ws/user`)

### Connection

```bash
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123"
```

### Automatic Listen Key Management

This stream implements **automatic listen key lifecycle management**:

1. **On Connect**: Server automatically creates a listen key via `POST /api/v1/userDataStream`
2. **Every 30 minutes**: Server automatically renews the listen key via `PUT /api/v1/userDataStream`
3. **On Disconnect**: Server automatically closes the listen key via `DELETE /api/v1/userDataStream`

**Client does not need to manage listen keys manually** when using this WebSocket endpoint.

### Message Flow

```
Client                          Server                      Binance
  |                               |                           |
  |-- WebSocket Upgrade -------->|                           |
  |                               |-- POST /userDataStream -->|
  |                               |<-- listenKey -------------|
  |<-- 101 Switching Protocols --|                           |
  |                               |-- Connect to user stream->|
  |                               |                           |
  |<----- ExecutionReport --------|<-- order update ---------|
  |<----- AccountPosition --------|<-- balance update -------|
  |                               |                           |
  |     [30 minutes later]        |                           |
  |                               |-- PUT /userDataStream --->|
  |                               |<-- OK --------------------|
  |                               |                           |
  |-- Close Frame -------------->|                           |
  |                               |-- DELETE /userDataStream->|
  |<-- Close Frame --------------|                           |
```

**Frequency**: Real-time on order/balance changes
**Upstream Source**: Binance user data stream (authenticated with API key)

### Message Types

The user data stream sends two types of events:

#### 1. Execution Report (Order Update)

Sent when order status changes (new, filled, canceled, etc.)

```json
{
  "e": "executionReport",
  "E": 1609459200000,
  "s": "BTCUSDT",
  "c": "x-A6SIDXVS",
  "S": "BUY",
  "o": "LIMIT",
  "f": "GTC",
  "q": "0.01",
  "p": "45000.00",
  "x": "NEW",
  "X": "NEW",
  "r": "",
  "i": 12345678,
  "l": "0.00",
  "z": "0.00",
  "L": "0.00",
  "n": "0.00",
  "N": null,
  "T": 1609459200000,
  "t": -1,
  "w": true,
  "m": false,
  "O": 1609459200000,
  "Z": "0.00",
  "Y": "0.00",
  "Q": "0.00"
}
```

**Key Fields**:
- `x`: Execution type (`"NEW"`, `"CANCELED"`, `"TRADE"`, `"EXPIRED"`)
- `X`: Order status (`"NEW"`, `"PARTIALLY_FILLED"`, `"FILLED"`, `"CANCELED"`)
- `i`: Order ID (use for matching with REST API orders)
- `z`: Cumulative filled quantity
- `Z`: Cumulative quote quantity

#### 2. Outbound Account Position (Balance Update)

Sent when account balances change:

```json
{
  "e": "outboundAccountPosition",
  "E": 1609459200000,
  "u": 1609459200000,
  "B": [
    {
      "a": "BTC",
      "f": "0.00100000",
      "l": "0.00000000"
    },
    {
      "a": "USDT",
      "f": "10000.00",
      "l": "0.00"
    }
  ]
}
```

**Key Fields**:
- `B`: Array of balance updates
- `a`: Asset symbol
- `f`: Free (available) balance
- `l`: Locked (in orders) balance

## Connection Lifecycle

### Normal Connection

```
1. Client sends WebSocket upgrade with Bearer token
2. Server validates token and connection limit
3. Server upgrades connection to WebSocket
4. Server starts streaming data
5. Client processes messages
6. Client sends Close frame (opcode 0x8)
7. Server sends Close frame
8. Connection closed
```

### Connection Limits

**Maximum**: 50 concurrent WebSocket connections

When limit is reached:
```http
HTTP/1.1 503 Service Unavailable
Retry-After: 60
Content-Type: application/json

{
  "error": "Maximum WebSocket connections reached",
  "status": 503
}
```

**Client should**:
- Wait for `Retry-After` seconds (60s)
- Retry connection
- Implement exponential backoff if persistent

### Disconnection Scenarios

#### 1. Normal Client Close

Client sends Close frame → Server acknowledges → Connection closed gracefully

#### 2. Server Close (Authentication Failure)

If token becomes invalid during connection:
```
Server → Close Frame (code 1008: Policy Violation)
```

#### 3. Network Error

Connection drops without Close frame → Server detects after ping timeout → Resources cleaned up

#### 4. Server Shutdown

Server sends Close frame to all clients → Waits for acknowledgments → Shuts down

## Error Handling

### Connection Errors

| Error | HTTP Status | Cause | Solution |
|-------|-------------|-------|----------|
| Missing token | 401 | No `Authorization` header | Add Bearer token |
| Invalid token | 401 | Token not in token store | Use valid token |
| Connection limit | 503 | 50+ active connections | Wait and retry |
| Invalid symbol | 400 | Symbol not found | Check symbol format |

### Runtime Errors

If Binance upstream connection fails:
- Server logs error
- Server sends Close frame with code 1011 (Internal Error)
- Client should reconnect after delay

### Ping/Pong

WebSocket ping/pong frames are handled automatically by the server:
- Server sends Ping frames every 30 seconds
- Client must respond with Pong frames
- Connection closed if Pong not received within 60 seconds

## Backpressure Handling

If client cannot process messages fast enough:

1. **Server-side buffering**: Messages buffered up to 100 pending
2. **Slow consumer detection**: If buffer full for >10 seconds
3. **Connection close**: Server sends Close frame (code 1008)
4. **Client should**: Increase processing speed or reduce subscription count

## Testing Examples

### Example 1: Monitor BTC Price

```bash
#!/bin/bash
# Watch BTCUSDT price updates

wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '.c + " @ " + (.E/1000|todate)'
```

Output:
```
45000.00 @ 2025-01-01T00:00:00Z
45001.50 @ 2025-01-01T00:00:01Z
45002.00 @ 2025-01-01T00:00:02Z
```

### Example 2: Monitor Order Book

```bash
#!/bin/bash
# Watch BTCUSDT order book updates

wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r '.b[0][0] + " / " + .a[0][0]'
```

Output:
```
44990.00 / 45000.00
44991.00 / 45000.00
44991.00 / 44999.00
```

### Example 3: Monitor User Orders

```bash
#!/bin/bash
# Watch order updates (requires Binance API key in env)

export BINANCE_API_KEY="your_api_key"
export BINANCE_API_SECRET="your_secret"

wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token_123" \
  | jq -r 'select(.e=="executionReport") | .X + " order " + (.i|tostring)'
```

Output:
```
NEW order 12345678
PARTIALLY_FILLED order 12345678
FILLED order 12345678
```

## Implementation Details

### Internal Architecture

```rust
// WebSocket handler flow
async fn handle_websocket(
    socket: WebSocket,
    stream_type: StreamType,
    token_store: TokenStore,
) {
    // 1. Validate token
    validate_bearer_token(&socket, &token_store)?;

    // 2. Check connection limit
    if active_connections >= 50 {
        return Err(ConnectionLimitError);
    }

    // 3. Connect to Binance upstream
    let binance_stream = connect_binance_stream(stream_type).await?;

    // 4. Stream messages
    loop {
        let msg = binance_stream.next().await?;
        socket.send(msg).await?;
    }
}
```

### User Data Stream Lifecycle

```rust
// Automatic listen key management
async fn handle_user_data_stream(socket: WebSocket) {
    // Create listen key
    let listen_key = create_listen_key().await?;

    // Start renewal task (every 30 minutes)
    let renewal_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1800)).await;
            renew_listen_key(&listen_key).await?;
        }
    });

    // Stream data
    let stream = connect_user_stream(&listen_key).await?;
    stream_to_client(socket, stream).await?;

    // Cleanup on disconnect
    renewal_task.abort();
    close_listen_key(&listen_key).await?;
}
```

### Connection Tracking

```rust
// Global connection counter
static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

async fn handle_new_connection() -> Result<(), Error> {
    // Check limit
    let current = ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
    if current >= 50 {
        ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
        return Err(ConnectionLimitError);
    }

    // Handle connection
    let _guard = ConnectionGuard; // RAII cleanup

    // ... stream data ...

    Ok(())
}

// RAII cleanup on disconnect
struct ConnectionGuard;
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
    }
}
```

## Protocol Compliance

### WebSocket RFC 6455 Compliance

✅ **Supported Features**:
- Text frames (opcode 0x1)
- Close frames (opcode 0x8)
- Ping frames (opcode 0x9)
- Pong frames (opcode 0xA)
- Frame fragmentation
- Connection upgrade handshake
- Sec-WebSocket-Key/Accept validation

❌ **Not Supported**:
- Binary frames (all data is JSON text)
- WebSocket extensions (compression, etc.)
- Subprotocols

### Security Considerations

1. **Authentication**: Bearer token required for all connections
2. **Token Storage**: Tokens hashed with SHA-256 (not stored plaintext)
3. **Connection Limits**: Prevents resource exhaustion (max 50)
4. **Timeout**: Idle connections closed after 60s without Pong
5. **Input Validation**: All incoming messages validated before processing

---

**References**:
- RFC 6455 (WebSocket Protocol): https://tools.ietf.org/html/rfc6455
- Binance WebSocket Streams: https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams
- axum WebSocket Handler: https://docs.rs/axum/latest/axum/extract/ws/index.html
- tokio-tungstenite: https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/
