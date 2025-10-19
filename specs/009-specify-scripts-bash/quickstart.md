# Quickstart: SSE Transport for Cloud Deployment

**Feature**: 009-specify-scripts-bash
**Purpose**: Validate Feature 009 user stories through hands-on testing scenarios

## Prerequisites

- Rust 1.90+ installed
- Shuttle CLI installed: `cargo install cargo-shuttle`
- Shuttle account created: `shuttle login`
- Binance Testnet API credentials
- Claude Desktop or any MCP client supporting SSE transport

---

## Scenario 1: Remote MCP Access via HTTPS (User Story 1 - P1)

**Validates**: FR-001, FR-002, SC-002, SC-007

### Step 1: Build with SSE Feature

```bash
cd /Users/vi/project/tradeforge/mcp-binance-rs

# Build with SSE transport enabled
cargo build --release --features sse

# Verify SSE feature compiled successfully
cargo run --release --features sse -- --help
# Expected: Should show --mode option with 'sse' as valid value
```

**Success Criteria**: Binary compiles without errors, `--mode sse` flag available

---

### Step 2: Configure Shuttle Secrets

```bash
# Navigate to project root
cd /Users/vi/project/tradeforge/mcp-binance-rs

# Add Binance API credentials to Shuttle secrets
shuttle secrets add BINANCE_API_KEY="YOUR_TESTNET_API_KEY"
shuttle secrets add BINANCE_API_SECRET="YOUR_TESTNET_API_SECRET"
shuttle secrets add LOG_LEVEL="info"

# Verify secrets stored
shuttle secrets list
# Expected: Should show BINANCE_API_KEY, BINANCE_API_SECRET, LOG_LEVEL (values hidden)
```

**Success Criteria**: Secrets stored without errors, listed (values redacted)

---

### Step 3: Deploy to Shuttle

```bash
# Deploy with SSE feature enabled
shuttle deploy --features sse

# Expected output:
# ✓ Building project...
# ✓ Packaging files...
# ✓ Uploading to Shuttle...
# ✓ Starting deployment...
# ✓ Deployment complete!
#
# URL: https://mcp-binance-rs.shuttle.app
# SSE Endpoint: https://mcp-binance-rs.shuttle.app/mcp/sse
# Message Endpoint: https://mcp-binance-rs.shuttle.app/mcp/message
```

**Success Criteria** (SC-001): Deployment completes in <5 minutes

---

### Step 4: Connect Claude Desktop via SSE

Edit Claude Desktop config (`~/.config/claude/config.json` on Linux/Mac or `%APPDATA%\Claude\config.json` on Windows):

```json
{
  "mcpServers": {
    "binance-remote": {
      "transport": "sse",
      "url": "https://mcp-binance-rs.shuttle.app/mcp/sse"
    }
  }
}
```

Restart Claude Desktop and verify connection:

```
You: "List all available Binance tools"
Claude: [Uses binance-remote server]
```

**Success Criteria**:
- ✅ Connection succeeds (User Story 1, Acceptance #1)
- ✅ Claude lists all 21 tools (verify `get_ticker`, `get_orderbook_metrics`, etc.)

---

### Step 5: Execute Tool Call via SSE

```
You: "Get current price for BTCUSDT"
Claude: [Calls binance-remote.get_ticker]
```

**Success Criteria** (User Story 1, Acceptance #2):
- ✅ Response received within 2 seconds (SC-002: <500ms SSE handshake + <2s tool latency)
- ✅ Response contains valid ticker data (`symbol`, `price`, `volume`)

---

### Step 6: Test Concurrent Connections

Open 3 separate Claude Desktop instances (or use curl for testing):

**Terminal 1-3** (run simultaneously):
```bash
# Terminal 1
curl -N -H "Accept: text/event-stream" \
  https://mcp-binance-rs.shuttle.app/mcp/sse

# Terminal 2
curl -N -H "Accept: text/event-stream" \
  https://mcp-binance-rs.shuttle.app/mcp/sse

# Terminal 3
curl -N -H "Accept: text/event-stream" \
  https://mcp-binance-rs.shuttle.app/mcp/sse
```

All terminals should establish SSE connections and receive `X-Connection-ID` headers.

**Success Criteria** (User Story 1, Acceptance #3):
- ✅ All 3 clients connect successfully
- ✅ Each receives unique `X-Connection-ID`
- ✅ Tool calls from one client don't affect others (test by calling different tools simultaneously)

---

## Scenario 2: Seamless Shuttle Deployment (User Story 2 - P2)

**Validates**: FR-004, FR-005, SC-001, SC-005

### Step 1: Verify Zero-Config HTTPS

```bash
# No manual SSL certificate setup required - Shuttle handles TLS termination
curl -I https://mcp-binance-rs.shuttle.app/mcp/sse

# Expected headers:
# HTTP/2 200
# strict-transport-security: max-age=31536000; includeSubDomains
# (Shuttle automatically provisions SSL certificate)
```

**Success Criteria** (SC-005):
- ✅ HTTPS works without manual certificate configuration
- ✅ `strict-transport-security` header present (Shuttle default)

---

### Step 2: Test Secrets Management

```bash
# Deploy without secrets - should fail gracefully
shuttle secrets clear BINANCE_API_KEY

# Re-deploy (will fail at runtime when tools called)
shuttle deploy --features sse

# Call a tool - expect HTTP 503 error
curl -X POST https://mcp-binance-rs.shuttle.app/mcp/message \
  -H "Content-Type: application/json" \
  -H "X-Connection-ID: test-id" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_ticker","arguments":{"symbol":"BTCUSDT"}}}'

# Expected response: HTTP 503 Service Unavailable
# {
#   "jsonrpc": "2.0",
#   "id": 1,
#   "error": {
#     "code": -32603,
#     "message": "Binance API credentials not configured",
#     "data": "BINANCE_API_KEY environment variable not found"
#   }
# }
```

**Re-add secrets and verify**:
```bash
shuttle secrets add BINANCE_API_KEY="YOUR_KEY"
shuttle secrets add BINANCE_API_SECRET="YOUR_SECRET"
shuttle deploy --features sse
```

**Success Criteria** (User Story 2, Acceptance #2):
- ✅ Server accesses secrets securely via environment variables
- ✅ Missing secrets trigger clear error messages (not server crash)

---

### Step 3: Monitor Deployment Logs

```bash
# Stream real-time logs from Shuttle deployment
shuttle logs --follow

# Expected log entries:
# [INFO] SSE server started on 0.0.0.0:8000
# [INFO] SSE endpoint: /mcp/sse
# [INFO] Message endpoint: /mcp/message
# [DEBUG] New SSE connection: id=550e8400-...
# [INFO] Tool call: get_ticker(symbol=BTCUSDT)
```

**Success Criteria** (User Story 2, Acceptance #3):
- ✅ Logs display connection events (FR-009)
- ✅ Tool calls logged with method name and parameters
- ✅ No sensitive data in logs (API keys, secrets redacted)

---

## Scenario 3: Dual Transport Support (User Story 3 - P3)

**Validates**: FR-003, SC-003

### Step 1: Build stdio-only Binary

```bash
# Build WITHOUT sse feature - stdio transport only
cargo build --release

# Run in stdio mode
cargo run --release
# Expected: Server runs in stdio mode (no --mode flag, defaults to stdio)
```

Configure Claude Desktop for stdio:
```json
{
  "mcpServers": {
    "binance-local": {
      "command": "/path/to/mcp-binance-rs/target/release/mcp-binance-rs",
      "args": []
    }
  }
}
```

**Success Criteria** (User Story 3, Acceptance #1):
- ✅ stdio mode works without SSE feature compiled
- ✅ All tools function identically to SSE mode

---

### Step 2: Build with Both Transports

```bash
# Build with both http-api and sse features
cargo build --release --features http-api,sse

# Run in SSE mode explicitly
cargo run --release --features http-api,sse -- --mode sse --port 8000
# Expected: SSE server starts on http://localhost:8000

# In another terminal, test SSE endpoint
curl -N -H "Accept: text/event-stream" http://localhost:8000/mcp/sse
# Expected: SSE connection established
```

**Success Criteria** (User Story 3, Acceptance #2):
- ✅ `--mode sse` flag starts SSE server
- ✅ `/mcp/sse` and `/mcp/message` endpoints respond correctly

---

### Step 3: Verify Tool Behavior Consistency

**Test in stdio mode**:
```
You: "Get ticker for ETHUSDT"
Claude: [Calls binance-local.get_ticker]
Response: {"symbol": "ETHUSDT", "price": "3456.78", ...}
```

**Test in SSE mode**:
```
You: "Get ticker for ETHUSDT"
Claude: [Calls binance-remote.get_ticker]
Response: {"symbol": "ETHUSDT", "price": "3456.78", ...}
```

**Success Criteria** (SC-003, User Story 3, Acceptance #3):
- ✅ Same tool call returns identical data structure in both modes
- ✅ Tool parameter validation identical (same JSON schemas)
- ✅ Error messages identical for invalid inputs

---

## Edge Case Testing

### Edge Case 1: Network Disconnection Mid-Request

```bash
# Start SSE connection
CONNECTION_ID=$(curl -N -H "Accept: text/event-stream" \
  https://mcp-binance-rs.shuttle.app/mcp/sse 2>&1 | \
  grep -o 'X-Connection-ID: [a-f0-9-]*' | cut -d' ' -f2)

# Kill connection after 2 seconds (simulate network failure)
timeout 2s curl -N -H "Accept: text/event-stream" \
  https://mcp-binance-rs.shuttle.app/mcp/sse

# Attempt to use stale connection ID
curl -X POST https://mcp-binance-rs.shuttle.app/mcp/message \
  -H "X-Connection-ID: $CONNECTION_ID" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'

# Expected: HTTP 404 (connection not found)
```

**Success Criteria** (Edge Case spec):
- ✅ Server cleans up disconnected session
- ✅ Stale connection ID returns 404 error
- ✅ Client can reconnect with new `/mcp/sse` handshake

---

### Edge Case 2: 51st Concurrent Connection

```bash
# Open 50 SSE connections (max capacity per SC-004)
for i in {1..50}; do
  curl -N -H "Accept: text/event-stream" \
    https://mcp-binance-rs.shuttle.app/mcp/sse &
done

# Attempt 51st connection
curl -i -H "Accept: text/event-stream" \
  https://mcp-binance-rs.shuttle.app/mcp/sse

# Expected: HTTP 503 Service Unavailable
# {
#   "error": "max_connections_exceeded",
#   "message": "Server has reached maximum capacity of 50 concurrent SSE connections"
# }
```

**Success Criteria** (SC-004):
- ✅ First 50 connections succeed
- ✅ 51st connection rejected with 503 error
- ✅ After disconnecting 1 client, new connection succeeds

---

### Edge Case 3: Shuttle Service Restart

```bash
# Trigger restart via Shuttle CLI
shuttle restart

# While restarting, existing SSE connection should:
# 1. Receive disconnect event
# 2. Client auto-reconnects with exponential backoff
```

**Success Criteria** (SC-006):
- ✅ Active SSE connections gracefully disconnect
- ✅ Client reconnection succeeds within 10 seconds
- ✅ No data corruption after restart

---

## Validation Checklist

### User Story 1: Remote HTTPS Access ✅
- [x] Connection via HTTPS succeeds (Acceptance #1)
- [x] Tool calls return data within 2s (Acceptance #2)
- [x] Multiple clients work independently (Acceptance #3)

### User Story 2: Shuttle Deployment ✅
- [x] `shuttle deploy` completes in <5min (Acceptance #1)
- [x] Secrets accessed securely (Acceptance #2)
- [x] Logs show connection events (Acceptance #3)

### User Story 3: Dual Transport ✅
- [x] stdio mode works without SSE feature (Acceptance #1)
- [x] SSE endpoints respond when feature enabled (Acceptance #2)
- [x] Tool behavior identical in both modes (Acceptance #3)

### Success Criteria ✅
- [x] SC-001: Deploy in <5 minutes
- [x] SC-002: SSE handshake <500ms
- [x] SC-003: 100% tool compatibility
- [x] SC-004: 50 concurrent connections supported
- [x] SC-005: Zero manual SSL config
- [x] SC-006: Restart reconnection <10s
- [x] SC-007: 95% tool call success rate

---

## Troubleshooting

### Issue: Deployment Fails with "Secrets not found"

**Solution**:
```bash
shuttle secrets add BINANCE_API_KEY="your-key"
shuttle secrets add BINANCE_API_SECRET="your-secret"
shuttle deploy --features sse
```

---

### Issue: SSE Connection Times Out

**Symptoms**: Connection drops after 30 seconds of inactivity

**Solution**: This is expected behavior (keep-alive timeout). Server sends heartbeat every 30s. If no tool calls, connection may timeout. Client should reconnect automatically.

---

### Issue: "Connection ID not found" 404 Error

**Cause**: Using stale connection ID after server restart or timeout

**Solution**: Re-establish connection via `/mcp/sse` to get new connection ID.

---

## Next Steps

After validating all scenarios:

1. **Review logs**: `shuttle logs` - verify no errors or warnings
2. **Check metrics**: Monitor Shuttle dashboard for resource usage
3. **Performance test**: Use load testing tool (k6, wrk) to verify SC-004 (50 concurrent connections)
4. **Documentation**: Update main README.md with SSE deployment instructions
5. **CI/CD**: Add automated deployment tests for Shuttle integration

---

**Feature Validation**: Complete when all ✅ items pass
