# Data Model: SSE Transport for Cloud Deployment

**Feature**: 009-specify-scripts-bash
**Date**: 2025-10-18

## Overview

This feature adds SSE transport protocol support without introducing new domain entities. SSE is a transport-layer concern that wraps existing MCP protocol messages and Binance data models. No new business entities are created.

## Entity Analysis from Spec

From spec.md Key Entities section, we identified:

1. **SSE Connection Session** - Transport metadata
2. **MCP Message** - Protocol-level messages
3. **Shuttle Configuration** - Deployment metadata

**Classification**: All three are **infrastructure/transport entities**, not domain entities. They do not represent business concepts like Orders, Trades, or Market Data.

## Transport Layer Entities

### 1. SSE Connection Session

**Purpose**: Represents an active SSE connection between client and server

**Lifecycle**: Created on `/mcp/sse` connection → Destroyed on disconnect

**Fields**:
- `connection_id`: Unique identifier (UUID v4)
- `client_addr`: IP address of connected client
- `connected_at`: Timestamp of connection establishment
- `last_activity`: Timestamp of last message exchange
- `user_agent`: Optional HTTP User-Agent header for debugging

**State Transitions**:
```
CONNECTING → ACTIVE → DISCONNECTED
             ↓
          TIMEOUT (if no activity >30s)
```

**Storage**: In-memory only (no persistence required)

**Relationships**:
- One SSE session → Many MCP messages (1:N)
- No relationship to Binance data models (transport is independent of business logic)

---

### 2. MCP Message

**Purpose**: JSON-RPC 2.0 message exchanged over SSE transport

**Lifecycle**: Created per client request → Sent to server → Response generated → Delivered via SSE

**Fields** (as defined by MCP protocol):
- `jsonrpc`: Always "2.0" (protocol version)
- `id`: Request identifier (string or number)
- `method`: RPC method name (e.g., "tools/call", "tools/list")
- `params`: Method-specific parameters (JSON object)
- `result`: Success response data (mutually exclusive with `error`)
- `error`: Error object with `code`, `message`, `data` fields

**Message Types**:
1. **Request**: Has `id`, `method`, `params`
2. **Response**: Has `id`, `result` OR `error`
3. **Notification**: Has `method`, `params`, no `id` (one-way message)

**Serialization**: JSON via `serde_json`

**Validation**: Handled by rmcp SDK (deserializes into `rmcp::protocol::JsonRpcMessage`)

**Relationships**:
- Belongs to one SSE Connection Session
- Contains one Tool Call or Tool Result
- No direct coupling to Binance models (protocol is transport-agnostic)

---

### 3. Shuttle Configuration

**Purpose**: Deployment metadata for Shuttle.dev platform

**Lifecycle**: Created once at deployment → Persisted in `Shuttle.toml` → Read at runtime

**Fields** (Shuttle.toml structure):
```toml
[project]
name = "mcp-binance-rs"  # Project identifier on Shuttle

[build]
builder = "shuttle"       # Build system

[runtime]
version = "0.56.0"        # Shuttle runtime version
```

**Secrets** (stored in Shuttle secrets store, not in code):
- `BINANCE_API_KEY`: Binance API key
- `BINANCE_API_SECRET`: Binance API secret
- `LOG_LEVEL`: Optional logging level override

**Storage**: File-based (`Shuttle.toml`) + Shuttle secrets service

**Validation**:
- Project name must be unique per Shuttle account
- Secrets must exist before deployment (validation at deploy time)

**Relationships**:
- Configuration consumed by runtime → No active runtime relationship to entities

---

## No New Domain Entities

**Important**: This feature does NOT introduce:
- ❌ New trading entities (orders, positions, balances)
- ❌ New market data entities (tickers, trades, order book)
- ❌ New user entities (accounts, permissions, sessions)

**Rationale**: SSE is a transport mechanism. Existing domain entities (from previous features) are reused without modification:
- `OrderBook` (from Feature 007) - unchanged
- `OrderBookMetrics` (from Feature 007) - unchanged
- `Ticker` (from Feature 001) - unchanged
- All MCP tool parameters/results - unchanged

---

## Data Flow Architecture

### SSE Connection Flow

```
Client → HTTPS GET /mcp/sse
  ↓
Server creates SSE Connection Session
  ↓
Server sends SSE event: connection-id, server-info
  ↓
Client → HTTPS POST /mcp/message (with connection-id header)
  ↓
Server parses MCP Message
  ↓
Server routes to existing tool handler (e.g., get_ticker)
  ↓
Server sends SSE event: tool result
  ↓
Client processes response
```

### Message Routing

```
SSE Transport Layer (NEW)
  ↓ deserialize MCP Message
rmcp ServerHandler (EXISTING)
  ↓ route to tool
Tool Implementation (EXISTING - get_ticker, get_orderbook_metrics, etc.)
  ↓ call Binance API
BinanceClient (EXISTING)
  ↓ HTTP/WebSocket request
Binance API
  ↓ market data response
Serialize to MCP Message
  ↓ send via SSE
Client
```

**Key Insight**: SSE layer sits ABOVE existing MCP server. No changes to tool implementations required.

---

## State Management

### In-Memory State

| Entity | Storage | Lifetime | Max Size |
|--------|---------|----------|----------|
| SSE Connection Sessions | HashMap<ConnectionId, Session> | Until disconnect | 50 connections (per spec SC-004) |
| Active WebSocket Streams | Managed by orderbook module | Until symbol unsubscribed | 20 symbols (existing limit) |
| Rate Limiter Tokens | GCRA state in-memory | Per-window (1 min) | O(1) per client |

### No Persistent State

- ❌ No database writes for SSE connections
- ❌ No session replay after server restart
- ❌ No connection state synchronization across instances

**Rationale**: Spec SC-006 allows <10s reconnection time after restart. Stateless design simplifies deployment.

---

## Error Handling

### SSE-Specific Errors

| Error Type | HTTP Status | SSE Event | Retry Strategy |
|------------|-------------|-----------|----------------|
| Invalid SSE handshake | 400 Bad Request | N/A | Client error - fix request |
| Missing connection-id header | 400 Bad Request | `error` event | Client error - fix request |
| Connection not found | 404 Not Found | `error` event | Reconnect via /mcp/sse |
| Server overloaded (>50 connections) | 503 Service Unavailable | N/A | Exponential backoff |
| Binance API key invalid | 503 Service Unavailable | `error` event | Fix configuration |

### MCP Protocol Errors

All existing MCP error codes reused:
- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid request (malformed JSON-RPC)
- `-32601`: Method not found (unknown tool)
- `-32602`: Invalid params (tool schema violation)
- `-32603`: Internal error (server exception)

**Handling**: rmcp SDK generates error responses automatically. SSE layer passes through unchanged.

---

## Validation Rules

### Connection Validation

1. **On `/mcp/sse` handshake**:
   - Accept: `text/event-stream` header must be present
   - Authorization: (MVP) None required / (Production) Bearer token validation
   - Rate limit: Max 50 concurrent connections (enforced in-memory)

2. **On `/mcp/message` POST**:
   - Content-Type: Must be `application/json`
   - Connection-ID header: Must match active SSE session
   - Body: Valid JSON-RPC 2.0 message (rmcp validates)

### Message Validation

Delegated to rmcp SDK:
- JSON-RPC 2.0 structure validation
- JSON Schema validation for tool parameters
- Method name validation against registered tools

**No custom validation** - reuse existing MCP server validation logic.

---

## Security Considerations

### Transport Security

- **Encryption**: HTTPS-only (Shuttle terminates TLS)
- **Authentication**: MVP = none, Production = OAuth2 Bearer tokens
- **Authorization**: All tools available to authenticated clients (no per-tool ACL in MVP)

### Data Protection

- **Binance API Keys**: Stored in Shuttle secrets, never in logs or responses
- **Error Messages**: Existing error sanitization applies (no sensitive data leakage)
- **Connection Metadata**: IP addresses logged at DEBUG level only

---

## Performance Characteristics

### Connection Overhead

- **Handshake**: <500ms (spec SC-002)
- **Keep-alive**: 30s SSE heartbeat (prevents proxy timeout)
- **Message latency**: Same as stdio mode (transport adds <50ms)

### Resource Usage

- **Memory per connection**: ~16KB (SSE buffer + connection metadata)
- **50 concurrent connections**: ~800KB memory overhead
- **No connection pooling**: Each client = dedicated SSE stream

---

## Testing Strategy

### Unit Tests

1. **SSE Session Lifecycle**:
   - Create session → verify UUID generated
   - Update last_activity → verify timestamp
   - Timeout detection → verify disconnect after 30s inactivity

2. **Message Serialization**:
   - Valid JSON-RPC request → deserialize successfully
   - Invalid JSON → parse error response
   - Missing required fields → validation error

### Integration Tests

1. **End-to-End SSE Flow**:
   - Connect to `/mcp/sse` → receive connection-id
   - POST to `/mcp/message` with tool call → receive SSE response
   - Disconnect → verify session cleanup

2. **Concurrent Connections**:
   - Open 50 connections → all succeed
   - Open 51st connection → 503 error

3. **Tool Compatibility**:
   - Call `get_ticker` via SSE → same response as stdio
   - Call `get_orderbook_metrics` via SSE → same response as stdio

**No mock Binance API** - use testnet API (per constitution testing policy)

---

## Summary

**Zero new domain entities** - this is purely a transport layer addition.

**Reuse existing entities**:
- All MCP tools unchanged
- All Binance data models unchanged
- All orderbook entities unchanged

**New infrastructure entities**:
- SSE Connection Session (in-memory, ephemeral)
- MCP Message (protocol envelope, handled by rmcp)
- Shuttle Configuration (deployment metadata, file-based)

**Design principle**: SSE wraps existing functionality without changing business logic.
