# Quickstart: MCP Server Foundation

**Feature**: MCP Server Foundation
**Purpose**: Validate that the implemented MCP server meets all acceptance criteria from [spec.md](./spec.md)
**Last Updated**: 2025-10-16

## Prerequisites

- Rust 1.75+ installed (`rustc --version`)
- Internet connection for Binance API access
- MCP-compatible client (Claude Desktop, or manual JSON-RPC testing)

## Installation

```bash
# Clone repository (if not already)
cd /path/to/mcp-binance-rs

# Build the server
cargo build --release

# Verify binary
./target/release/mcp-binance-server --version
```

## Configuration

### Environment Variables

```bash
# Optional: Set Binance API credentials
# (Not required for get_server_time, but server validates presence)
export BINANCE_API_KEY="your_api_key_here"
export BINANCE_SECRET_KEY="your_secret_key_here"

# Optional: Enable debug logging to see credential loading
export RUST_LOG=debug
```

**Note**: For foundation phase, credentials are optional. Server starts without them and logs a warning.

## Usage Scenarios

### Scenario 1: MCP Initialization (User Story 1)

**Goal**: Verify AI assistant can connect and discover tools

**Steps**:

1. Start the server (stdio transport):
```bash
# Run with default logging
./target/release/mcp-binance-server

# Or with debug logs
RUST_LOG=debug ./target/release/mcp-binance-server
```

2. Send MCP initialize request (via stdin):
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"TestClient","version":"1.0.0"}}}
```

3. **Expected Response** (via stdout):
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {
        "listChanged": false
      }
    },
    "serverInfo": {
      "name": "mcp-binance-server",
      "version": "0.1.0"
    }
  }
}
```

4. Send initialized notification:
```json
{"jsonrpc":"2.0","method":"notifications/initialized"}
```

5. Request tools list:
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

6. **Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "get_server_time",
        "description": "Returns the current Binance server time in milliseconds...",
        "inputSchema": {
          "type": "object",
          "properties": {},
          "additionalProperties": false
        }
      }
    ]
  }
}
```

**Success Criteria** (SC-001):
- ✅ Initialization completes in < 500ms
- ✅ Server responds with protocol version "2024-11-05"
- ✅ tools/list returns at least one tool (get_server_time)
- ✅ JSON Schema present for tool

---

### Scenario 2: Get Binance Server Time (User Story 2)

**Goal**: Verify connectivity and time synchronization

**Steps**:

1. (Assuming server initialized from Scenario 1)

2. Call get_server_time tool:
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_server_time","arguments":{}}}
```

3. **Expected Response** (success):
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"serverTime\":1609459200000}"
      }
    ]
  }
}
```

4. Verify timestamp is reasonable:
```bash
# Convert milliseconds to date (should be current time)
date -r $((1609459200000 / 1000))
```

5. Check logs for time synchronization:
```
INFO mcp_binance_server: Binance server time: 2025-10-16T12:00:00Z (local offset: +5ms)
```

**Success Criteria** (SC-002):
- ✅ Response received in < 100ms
- ✅ serverTime is valid Unix timestamp in milliseconds
- ✅ Time is within ±5 seconds of local system time
- ✅ No errors returned

---

### Scenario 3: Secure Credential Management (User Story 3)

**Goal**: Verify credentials are loaded securely

**Test A: With Credentials**

```bash
# Set valid (or test) credentials
export BINANCE_API_KEY="test_api_key_1234567890abcdef"
export BINANCE_SECRET_KEY="test_secret_key_abcdef1234567890"

# Start with INFO logging
RUST_LOG=info ./target/release/mcp-binance-server
```

**Expected Log Output**:
```
INFO mcp_binance_server: API credentials configured (key: test_...cdef)
INFO mcp_binance_server: MCP server starting...
```

**Verify**:
- ✅ Log shows "credentials configured" without full key
- ✅ Only first 4 and last 4 chars visible (test_...cdef)
- ✅ Secret key never appears in logs

**Test B: Without Credentials**

```bash
# Unset credentials
unset BINANCE_API_KEY BINANCE_SECRET_KEY

# Start server
RUST_LOG=info ./target/release/mcp-binance-server
```

**Expected Log Output**:
```
WARN mcp_binance_server: No API credentials configured; authenticated features disabled
INFO mcp_binance_server: MCP server starting...
```

**Verify**:
- ✅ Server starts successfully
- ✅ Warning logged about missing credentials
- ✅ get_server_time still works (public endpoint)

**Test C: Debug Logging with Credentials**

```bash
# Set credentials again
export BINANCE_API_KEY="test_key" BINANCE_SECRET_KEY="test_secret"

# Start with DEBUG logging
RUST_LOG=debug ./target/release/mcp-binance-server
```

**Expected Log Output**:
```
DEBUG mcp_binance_server::config: [SENSITIVE DATA] Loading API key: test_key
DEBUG mcp_binance_server::config: [SENSITIVE DATA] Loading secret key: test_secret
INFO mcp_binance_server: API credentials configured (key: test_...key)
```

**Verify**:
- ✅ Full credentials visible in DEBUG logs only
- ✅ "[SENSITIVE DATA]" prefix present
- ✅ INFO logs still mask credentials

**Success Criteria** (SC-004):
- ✅ Zero credential exposure at INFO/WARN levels
- ✅ Debug logs clearly marked as sensitive
- ✅ Error messages never contain credentials

---

### Scenario 4: Error Handling

**Test A: Network Failure**

```bash
# Simulate network failure (block Binance API)
sudo iptables -A OUTPUT -d api.binance.com -j DROP  # Linux
# Or disconnect network

# Call get_server_time
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_server_time","arguments":{}}}
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Connection error: Failed to connect to Binance API after 3 retries. Please check your internet connection."
      }
    ],
    "isError": true
  }
}
```

**Verify**:
- ✅ `isError: true` flag set
- ✅ User-friendly error message
- ✅ No sensitive data in error
- ✅ Error type: "connection_error"

**Test B: Rate Limiting** (requires many requests)

```bash
# Spam requests to trigger rate limit
for i in {1..1500}; do
  echo '{"jsonrpc":"2.0","id":'$i',"method":"tools/call","params":{"name":"get_server_time","arguments":{}}}' | ./target/release/mcp-binance-server
done
```

**Expected Response** (after ~1200 requests):
```json
{
  "jsonrpc": "2.0",
  "id": 1201,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Rate limit exceeded: Too many requests to Binance API. Retry after 60 seconds."
      }
    ],
    "isError": true
  }
}
```

**Verify**:
- ✅ Rate limit error returned
- ✅ Retry-after information included
- ✅ No server crash
- ✅ Exponential backoff applied (logs show retry delays)

**Success Criteria** (SC-005):
- ✅ Graceful error recovery in 100% of cases
- ✅ User-friendly error messages
- ✅ Structured error types

---

### Scenario 5: Integration with Claude Desktop

**Goal**: Validate MCP protocol compliance (SC-006)

**Steps**:

1. Add server to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):
```json
{
  "mcpServers": {
    "binance": {
      "command": "/path/to/mcp-binance-rs/target/release/mcp-binance-server",
      "env": {
        "BINANCE_API_KEY": "your_key_here",
        "BINANCE_SECRET_KEY": "your_secret_here",
        "RUST_LOG": "info"
      }
    }
  }
}
```

2. Restart Claude Desktop

3. In chat, ask: "What time is it on the Binance servers?"

4. **Expected Behavior**:
- Claude discovers get_server_time tool
- Claude calls the tool
- Claude receives timestamp and formats response
- Response shows current time

**Verify**:
- ✅ Tool appears in Claude's tool list
- ✅ Tool execution succeeds
- ✅ Response is coherent and accurate
- ✅ No error messages

**Success Criteria** (SC-006):
- ✅ Successful integration with Claude Desktop
- ✅ MCP protocol fully compliant

---

## Performance Validation

### Startup Time (SC-007)

```bash
# Measure startup time
time (./target/release/mcp-binance-server --help)
```

**Expected**: < 1 second

### Tool Execution Latency (SC-002)

```bash
# Measure get_server_time latency
time (echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"Test","version":"1.0.0"}}}' | ./target/release/mcp-binance-server)
```

**Expected**: < 100ms for get_server_time

### Memory Footprint

```bash
# Monitor memory usage
ps aux | grep mcp-binance-server

# Or with detailed metrics
valgrind --tool=massif ./target/release/mcp-binance-server
```

**Expected**: < 50MB memory usage

---

## Troubleshooting

### Server doesn't start

**Check**:
- Rust 1.75+ installed: `rustc --version`
- Binary compiled: `cargo build --release`
- No port conflicts (stdio only, no ports used)

### Tool call fails

**Check**:
- Internet connection active
- Binance API reachable: `curl https://api.binance.com/api/v3/time`
- Logs for detailed error: `RUST_LOG=debug ./target/release/mcp-binance-server`

### Credentials not loading

**Check**:
- Environment variables set: `echo $BINANCE_API_KEY`
- No whitespace in env vars
- Server restarted after setting vars

### Claude Desktop integration fails

**Check**:
- Config file syntax correct (valid JSON)
- Binary path absolute (not relative)
- Permissions: `chmod +x ./target/release/mcp-binance-server`
- Restart Claude Desktop after config changes

---

## Success Checklist

Use this checklist to validate all acceptance criteria are met:

### User Story 1: AI Assistant Initialization
- [ ] Server starts via stdio transport
- [ ] MCP initialize request returns server info and capabilities
- [ ] tools/list returns get_server_time tool
- [ ] JSON Schema present for tool
- [ ] Initialization completes in < 500ms

### User Story 2: Binance Time Synchronization
- [ ] get_server_time tool callable
- [ ] Returns valid Unix timestamp in milliseconds
- [ ] Response time < 100ms
- [ ] Network errors return structured error
- [ ] Time synchronization logged

### User Story 3: Secure API Key Management
- [ ] Credentials loaded from environment variables
- [ ] INFO logs mask credentials (only show first 4 + last 4 chars)
- [ ] Server starts without credentials (with warning)
- [ ] Debug logs show full credentials with [SENSITIVE DATA] prefix
- [ ] Error messages never expose credentials

### Edge Cases
- [ ] Rate limit errors handled gracefully
- [ ] Invalid JSON returns parse error
- [ ] Stdio disconnect triggers graceful shutdown
- [ ] Whitespace in env vars trimmed
- [ ] get_server_time during init returns "not ready" error

### Performance
- [ ] Startup time < 1 second
- [ ] get_server_time latency < 100ms
- [ ] Memory usage < 50MB

### Integration
- [ ] Works with Claude Desktop
- [ ] MCP protocol fully compliant

---

## Next Steps

After validating all scenarios:

1. Generate tasks: `/speckit.tasks`
2. Implement foundation: `/speckit.implement`
3. Run this quickstart again to verify implementation
4. Proceed to feature 002 (spot market data tools)

---

## Automation

Create a test script for CI/CD:

```bash
#!/bin/bash
# test_quickstart.sh

set -e

echo "Building server..."
cargo build --release

echo "Testing MCP initialization..."
# Add automated JSON-RPC test sequence here

echo "Testing get_server_time..."
# Add tool call test

echo "Testing credential loading..."
# Add env var tests

echo "All tests passed!"
```

Run with: `bash test_quickstart.sh`
