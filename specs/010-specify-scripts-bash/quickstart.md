# Quickstart: Streamable HTTP Transport Cleanup

**Feature**: 010-specify-scripts-bash | **Date**: 2025-10-18

## Overview

This quickstart verifies that the cleanup refactoring successfully removed legacy SSE code while preserving Streamable HTTP functionality.

## Prerequisites

- Binance MCP server running with `--mode sse` flag
- `curl` or equivalent HTTP client
- Port 8000 available (or configured port)

## Test Scenarios

### Scenario 1: Initialize Session (P1 - ChatGPT Integration)

**Objective**: Verify POST /mcp with `initialize` creates session and returns `Mcp-Session-Id` header

**Steps**:
```bash
# Start server (if not running)
cargo run --features sse -- --mode sse --port 8000

# Initialize session
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -i \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "test", "version": "1.0"}
    },
    "id": 1
  }'
```

**Expected Result**:
- HTTP 200 OK
- Response header: `Mcp-Session-Id: <uuid>`
- Response body contains:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
      "protocolVersion": "2024-11-05",
      "capabilities": {"tools": {}},
      "serverInfo": {"name": "Binance MCP Server", "version": "..."}
    }
  }
  ```

**Save session ID for next tests**: Extract UUID from `Mcp-Session-Id` header

---

### Scenario 2: List Tools with Valid Session (P1 - ChatGPT Integration)

**Objective**: Verify POST /mcp with `tools/list` requires and validates `Mcp-Session-Id` header

**Steps**:
```bash
# Replace <SESSION_ID> with UUID from Scenario 1
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 2
  }'
```

**Expected Result**:
- HTTP 200 OK
- Response body contains tools array:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 2,
    "result": {
      "tools": [
        {"name": "search", "description": "...", "inputSchema": {...}},
        {"name": "fetch", "description": "...", "inputSchema": {...}},
        {"name": "get_ticker", ...},
        {"name": "get_exchange_info", ...},
        {"name": "get_klines", ...}
      ]
    }
  }
  ```

---

### Scenario 3: Call Tool with Valid Session (P1 - ChatGPT Integration)

**Objective**: Verify POST /mcp with `tools/call` executes tools and returns MCP content array format

**Steps**:
```bash
# Replace <SESSION_ID> with UUID from Scenario 1
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "search",
      "arguments": {"query": "BTC"}
    },
    "id": 3
  }'
```

**Expected Result**:
- HTTP 200 OK
- Response body contains MCP content array:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 3,
    "result": {
      "content": [
        {
          "type": "text",
          "text": "{\"results\": [{\"id\": \"BTCUSDT\", \"title\": \"BTC/USDT\", ...}]}"
        }
      ]
    }
  }
  ```

---

### Scenario 4: Missing Session ID Error (P3 - Error Responses)

**Objective**: Verify non-initialize requests without `Mcp-Session-Id` header return 400 error

**Steps**:
```bash
# Send tools/list WITHOUT session header
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -i \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 4
  }'
```

**Expected Result**:
- HTTP 400 Bad Request
- Response body:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 4,
    "error": {
      "code": -32002,
      "message": "Missing Mcp-Session-Id header"
    }
  }
  ```

---

### Scenario 5: Invalid Session ID Error (P3 - Error Responses)

**Objective**: Verify invalid session ID returns 404 error

**Steps**:
```bash
# Send request with fake session ID
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: invalid-uuid-12345" \
  -i \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 5
  }'
```

**Expected Result**:
- HTTP 404 Not Found
- Response body:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 5,
    "error": {
      "code": -32001,
      "message": "Session not found or expired"
    }
  }
  ```

---

### Scenario 6: Old GET Endpoint Removed (P2 - Maintainable Codebase)

**Objective**: Verify GET /sse endpoint no longer exists (returns 404 or 405)

**Steps**:
```bash
# Try old SSE handshake endpoint
curl -X GET http://localhost:8000/sse -i
```

**Expected Result**:
- HTTP 404 Not Found (endpoint removed from router)
- No `X-Connection-ID` header in response

**Note**: This test may return 405 Method Not Allowed if route exists but GET is disabled. Either is acceptable as long as the old handshake doesn't work.

---

### Scenario 7: Backward Compatibility Endpoint (P2 - Maintainable Codebase)

**Objective**: Verify POST /messages still works (backward compatibility alias)

**Steps**:
```bash
# Test backward compatibility endpoint
curl -X POST http://localhost:8000/messages \
  -H "Content-Type: application/json" \
  -i \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "test", "version": "1.0"}
    },
    "id": 6
  }'
```

**Expected Result**:
- HTTP 200 OK
- Response header: `Mcp-Session-Id: <uuid>` (same as POST /mcp)
- Identical behavior to POST /mcp (same handler function)

---

### Scenario 8: Session Timeout (Edge Case)

**Objective**: Verify session expires after 5 seconds of inactivity

**Steps**:
```bash
# 1. Create session
SESSION_ID=$(curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -s \
  -D - \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
  | grep -i "mcp-session-id" | cut -d' ' -f2 | tr -d '\r')

# 2. Wait 6 seconds (exceeds 5s timeout)
sleep 6

# 3. Try to use expired session
curl -X POST http://localhost:8000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -i \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 7
  }'
```

**Expected Result**:
- HTTP 404 Not Found
- Error code -32001 (session expired)

---

### Scenario 9: ChatGPT Connector Integration (P1 - End-to-End)

**Objective**: Verify ChatGPT can connect and execute tools without GET handshake

**Steps**:
1. Configure ChatGPT connector with MCP server URL: `http://localhost:8000/mcp`
2. Ask ChatGPT: "Search for BTC trading pairs on Binance"
3. Observe ChatGPT calling `search` tool
4. Ask ChatGPT: "Get details for BTCUSDT"
5. Observe ChatGPT calling `fetch` tool

**Expected Result**:
- ChatGPT successfully connects using POST /mcp (no GET handshake)
- `search` tool returns BTC trading pairs
- `fetch` tool returns BTCUSDT market data with order book
- No errors related to missing handshake or invalid headers

**Note**: This is the ultimate integration test - if ChatGPT works, the cleanup was successful.

---

## Success Criteria Verification

After running all scenarios, verify:

- ✅ **SC-001**: ChatGPT successfully connects and executes tools without requiring GET handshake (Scenario 9)
- ✅ **SC-002**: Codebase contains zero references to old `sse_handshake` GET endpoint pattern (verify with `rg "sse_handshake|X-Connection-ID"`)
- ✅ **SC-003**: All integration tests pass using only POST /mcp endpoint (Scenarios 1-3)
- ✅ **SC-004**: Session management code is 40% smaller (measure lines of code in handlers_simple.rs)
- ✅ **SC-005**: Developer can understand transport flow in under 5 minutes of code review (subjective, but cleaner code helps)

## Cleanup Verification Commands

### Check for removed code references
```bash
# Should return 0 matches after cleanup
rg "sse_handshake" src/
rg "X-Connection-ID" src/
```

### Verify only Mcp-Session-Id remains
```bash
# Should find matches only in message_post handler
rg "Mcp-Session-Id" src/
```

### Count lines removed
```bash
# Before cleanup: handlers_simple.rs ~526 lines
# After cleanup: handlers_simple.rs ~400 lines (estimate)
wc -l src/transport/sse/handlers_simple.rs
```

## Troubleshooting

### Session ID not returned
- **Problem**: POST /mcp initialize doesn't return `Mcp-Session-Id` header
- **Check**: Verify `is_initialize` logic correctly detects `method == "initialize"`
- **Check**: Verify response headers are set after JSON-RPC response creation

### 404 on valid session
- **Problem**: Valid session returns "session not found" error
- **Check**: Verify session timeout hasn't triggered (default 5s)
- **Check**: Verify session manager correctly stores session after initialize
- **Check**: Check server logs for session creation/lookup traces

### ChatGPT connector fails
- **Problem**: ChatGPT shows "Unable to connect to MCP server"
- **Check**: Verify server is running with `--mode sse` flag
- **Check**: Verify firewall allows port 8000 (or configured port)
- **Check**: Test with curl first (Scenarios 1-3) to isolate ChatGPT vs server issues

### Old GET endpoint still works
- **Problem**: GET /sse returns 200 instead of 404
- **Check**: Verify router configuration removed `.route("/sse", get(sse_handshake))`
- **Check**: Verify `sse_handshake` function is completely removed from handlers_simple.rs
- **Check**: Rebuild and restart server to ensure old binary isn't running

## Next Steps

After successful verification:
1. ✅ Commit cleanup changes to feature branch
2. ✅ Run `cargo test --features sse` to ensure integration tests pass
3. ✅ Open PR to merge `010-specify-scripts-bash` branch to main
4. ✅ Deploy to Shuttle.dev and verify production behavior
5. ✅ Update CHANGELOG documenting removed endpoints and migration path
