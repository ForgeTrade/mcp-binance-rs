# HTTP Endpoints Contract: Streamable HTTP Transport

**Feature**: 010-specify-scripts-bash | **Date**: 2025-10-18

## Overview

This document defines the HTTP endpoint contract for MCP Streamable HTTP transport. This is a **cleanup refactoring** that removes deprecated endpoints while preserving the working Streamable HTTP implementation.

## Endpoints After Cleanup

### POST /mcp (Primary Endpoint)

**Purpose**: Handle all MCP JSON-RPC requests via Streamable HTTP transport

**Request Headers**:
- `Content-Type: application/json` (required)
- `Accept: application/json` or `Accept: text/event-stream` (optional, default: application/json)
- `Mcp-Session-Id: <uuid>` (required for all methods except `initialize`)

**Request Body**: JSON-RPC 2.0 request
```json
{
  "jsonrpc": "2.0",
  "method": "initialize|tools/list|tools/call|...",
  "params": { ... },
  "id": 1
}
```

**Response Headers**:
- `Content-Type: application/json` (or `text/event-stream` if requested)
- `Mcp-Session-Id: <uuid>` (only for `initialize` method response)

**Response Body**: JSON-RPC 2.0 response
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

**Behavior**:
- **First request (`initialize` method)**: Creates new session, returns `Mcp-Session-Id` header
- **Subsequent requests**: Validates `Mcp-Session-Id` header, updates activity timestamp
- **Missing session ID**: Returns 400 Bad Request with error code -32002
- **Invalid session ID**: Returns 404 Not Found with error code -32001
- **Session limit exceeded**: Returns 503 Service Unavailable with error code -32000

### POST /messages (Backward Compatibility)

**Purpose**: Alias to POST /mcp for backward compatibility

**Contract**: Identical to POST /mcp (same handler function)

**Rationale**: Provides migration path for any clients still using old `/messages` endpoint

**Deprecation**: May be removed in future release after monitoring usage

### GET /health (Health Check)

**Purpose**: Service health monitoring

**Request**: No parameters

**Response**: Plain text "OK"

**Status Code**: 200 OK

**Rationale**: Unchanged from previous implementation

## Removed Endpoints

### ❌ GET /sse (REMOVED)

**Old Purpose**: SSE handshake that returned `X-Connection-ID` header

**Why Removed**: Streamable HTTP uses POST-only protocol, no GET handshake needed

**Migration Path**: Use POST /mcp with `initialize` method instead

### ❌ GET /mcp/sse (REMOVED)

**Old Purpose**: Custom SSE handshake endpoint

**Why Removed**: Non-standard path, deprecated SSE pattern

**Migration Path**: Use POST /mcp with `initialize` method instead

### ❌ POST /mcp/message (REMOVED)

**Old Purpose**: Custom message endpoint

**Why Removed**: Non-standard path, redundant with POST /mcp

**Migration Path**: Use POST /mcp directly

## Error Responses

All errors follow JSON-RPC 2.0 format with Streamable HTTP-specific codes:

### 400 Bad Request (Missing Session ID)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32002,
    "message": "Missing Mcp-Session-Id header"
  }
}
```

### 404 Not Found (Invalid Session ID)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Session not found or expired"
  }
}
```

### 503 Service Unavailable (Session Limit)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Maximum concurrent sessions reached (50)"
  }
}
```

## Session Lifecycle

```
Client                              Server
  |                                   |
  |--- POST /mcp (initialize) ------->|
  |<-- 200 OK + Mcp-Session-Id: abc --|
  |                                   |
  |--- POST /mcp (tools/list) ------->|
  |    Header: Mcp-Session-Id: abc    |
  |<-- 200 OK (tools list) ------------|
  |                                   |
  |--- POST /mcp (tools/call) ------->|
  |    Header: Mcp-Session-Id: abc    |
  |<-- 200 OK (tool result) ----------|
  |                                   |
  [5s inactivity timeout]             |
  |                                   |
  |--- POST /mcp (tools/call) ------->|
  |    Header: Mcp-Session-Id: abc    |
  |<-- 404 Not Found (session expired)|
```

## Implementation Notes

### No API Contract Changes

This cleanup **does not change** the MCP protocol contract:
- JSON-RPC 2.0 message format unchanged
- MCP method names unchanged (`initialize`, `tools/list`, `tools/call`)
- Session timeout logic unchanged (5 seconds)
- Maximum session limit unchanged (50 concurrent)

### Only Endpoint Routing Changes

What changed:
- ✅ POST /mcp remains (primary endpoint)
- ✅ POST /messages remains (backward compatibility)
- ❌ GET /sse removed (deprecated handshake)
- ❌ GET /mcp/sse removed (custom path)
- ❌ POST /mcp/message removed (custom path)

### Header Changes

Before (SSE):
- GET /sse → `X-Connection-ID: <uuid>` response header
- POST /messages with `X-Connection-ID: <uuid>` request header

After (Streamable HTTP):
- POST /mcp with `initialize` → `Mcp-Session-Id: <uuid>` response header
- POST /mcp with other methods → `Mcp-Session-Id: <uuid>` request header

## Verification

**Success Criteria**:
1. POST /mcp with `initialize` creates session and returns `Mcp-Session-Id`
2. POST /mcp with `tools/list` requires valid `Mcp-Session-Id`
3. POST /mcp with `tools/call` executes tools with valid session
4. GET /sse returns 404 (endpoint removed)
5. ChatGPT connector successfully uses POST /mcp without GET handshake

**Test Scenarios**: See [quickstart.md](quickstart.md) for detailed test steps
