# Data Model: Streamable HTTP Transport Cleanup

**Feature**: 010-specify-scripts-bash | **Date**: 2025-10-18

## Overview

This feature is a code cleanup refactoring with **no data model changes**. All existing entities remain unchanged.

## Existing Entities (Unchanged)

### Session

**Purpose**: Represents an active MCP client connection

**Fields**:
- `session_id`: String (UUID format) - Unique session identifier
- `client_addr`: SocketAddr - Client IP address for logging
- `created_at`: Instant - Session creation timestamp
- `last_activity`: Instant - Last request timestamp (for timeout detection)

**Validation Rules**:
- `session_id` MUST be unique across all active sessions
- Maximum 50 concurrent sessions (enforced by SessionManager)
- Session expires after 5 seconds of inactivity (no requests)

**State Transitions**:
```
[Not Exists] --initialize--> [Active] --timeout/explicit close--> [Expired]
```

**Storage**: In-memory HashMap<String, Session> (no persistence)

**No Changes**: This cleanup removes code referencing `X-Connection-ID` header but doesn't modify the Session struct itself.

### JSON-RPC Request (MCP Protocol)

**Purpose**: Standard JSON-RPC 2.0 message from client to server

**Fields**:
- `jsonrpc`: String (always "2.0")
- `method`: String (e.g., "initialize", "tools/list", "tools/call")
- `params`: Object (method-specific parameters)
- `id`: Number or String (request identifier)

**Validation Rules**:
- `jsonrpc` MUST be exactly "2.0"
- `method` MUST be a valid MCP method name
- `id` MUST be present for requests requiring responses

**No Changes**: This cleanup doesn't affect JSON-RPC message structure.

### JSON-RPC Response (MCP Protocol)

**Purpose**: Standard JSON-RPC 2.0 response from server to client

**Fields**:
- `jsonrpc`: String (always "2.0")
- `id`: Number or String (matches request id)
- `result`: Object (success case)
- `error`: Object (error case, mutually exclusive with result)
  - `code`: Number (e.g., -32000, -32001, -32002)
  - `message`: String (human-readable error description)

**Validation Rules**:
- Exactly one of `result` or `error` MUST be present
- Error codes MUST follow JSON-RPC 2.0 and MCP Streamable HTTP spec

**No Changes**: This cleanup doesn't affect response structure, only which HTTP headers are included.

## What This Cleanup Changes

### Header Changes (Not Data Model)

**Before (Legacy SSE)**:
- GET `/sse` returns `X-Connection-ID: <uuid>` header
- POST `/messages` requires `X-Connection-ID: <uuid>` header

**After (Streamable HTTP)**:
- POST `/mcp` with `initialize` method returns `Mcp-Session-Id: <uuid>` header
- POST `/mcp` with other methods requires `Mcp-Session-Id: <uuid>` header

**Impact**: Session lookup changes from `headers.get("X-Connection-ID")` to `headers.get("Mcp-Session-Id")`, but the Session entity itself is unchanged.

### Endpoint Changes (Not Data Model)

**Removed**:
- GET `/sse` endpoint (handshake)
- GET `/mcp/sse` endpoint (custom path)
- `sse_handshake()` handler function

**Kept**:
- POST `/mcp` endpoint (primary)
- POST `/messages` endpoint (backward compatibility alias)
- `message_post()` handler function

**Impact**: Routing configuration changes, no data structures affected.

## Summary

This feature has **zero data model changes**. It's purely a code cleanup:
- Remove deprecated GET handshake endpoint
- Switch from `X-Connection-ID` to `Mcp-Session-Id` header (same session data)
- Consolidate dual-endpoint logic to single POST handler

All MCP protocol entities (Session, JSON-RPC Request/Response) remain unchanged.
