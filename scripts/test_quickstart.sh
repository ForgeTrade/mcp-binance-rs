#!/bin/bash
# Quickstart Validation Script for MCP Binance Server
# Automates scenarios from specs/001-mcp-server-foundation/quickstart.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
PASS=0
FAIL=0

# Helper functions
print_header() {
    echo ""
    echo "========================================="
    echo "$1"
    echo "========================================="
}

print_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASS++))
}

print_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAIL++))
}

print_info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

# Check prerequisites
print_header "Checking Prerequisites"

if ! command -v cargo &> /dev/null; then
    print_fail "cargo not found. Install Rust 1.90+"
    exit 1
fi

RUST_VERSION=$(rustc --version | awk '{print $2}' | cut -d'.' -f1,2)
if [[ $(echo "$RUST_VERSION >= 1.90" | bc -l) -eq 1 ]]; then
    print_pass "Rust version $RUST_VERSION meets requirement (>=1.90)"
else
    print_fail "Rust version $RUST_VERSION < 1.90"
    exit 1
fi

print_pass "Prerequisites check complete"

# Build the server
print_header "Building Server (Scenario Setup)"

if cargo build --release 2>&1 | grep -q "Finished"; then
    print_pass "Release build successful"
else
    print_fail "Release build failed"
    exit 1
fi

if [ -f "./target/release/mcp-binance-server" ]; then
    print_pass "Binary exists at target/release/mcp-binance-server"
else
    print_fail "Binary not found"
    exit 1
fi

# Scenario 1: MCP Initialization
print_header "Scenario 1: MCP Initialization"

print_info "Starting server and sending initialize request..."

# Use a temporary file for server output
TMP_OUT=$(mktemp)
TMP_ERR=$(mktemp)

# Start server in background
./target/release/mcp-binance-server > "$TMP_OUT" 2> "$TMP_ERR" &
SERVER_PID=$!

sleep 1

# Check if server is running
if ps -p $SERVER_PID > /dev/null; then
    print_pass "Server started (PID: $SERVER_PID)"
else
    print_fail "Server failed to start"
    cat "$TMP_ERR"
    rm "$TMP_OUT" "$TMP_ERR"
    exit 1
fi

# Send initialize request
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"TestClient","version":"1.0.0"}}}'
echo "$INIT_REQUEST" | ./target/release/mcp-binance-server > "$TMP_OUT" 2> "$TMP_ERR" &
INIT_PID=$!

sleep 2

# Check response
if grep -q '"protocolVersion":"2024-11-05"' "$TMP_OUT"; then
    print_pass "Protocol version 2024-11-05 in response"
else
    print_fail "Protocol version not found in response"
    cat "$TMP_OUT"
fi

if grep -q '"name":"mcp-binance-server"' "$TMP_OUT"; then
    print_pass "Server name in response"
else
    print_fail "Server name not found"
fi

if grep -q '"tools"' "$TMP_OUT"; then
    print_pass "Tools capability advertised"
else
    print_fail "Tools capability not found"
fi

# Cleanup
kill $SERVER_PID 2>/dev/null || true
kill $INIT_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true
wait $INIT_PID 2>/dev/null || true

# Scenario 2: get_server_time Tool
print_header "Scenario 2: Get Binance Server Time"

print_info "Testing get_server_time tool..."

# Start fresh server
./target/release/mcp-binance-server > "$TMP_OUT" 2> "$TMP_ERR" &
SERVER_PID=$!
sleep 1

# Send initialize + tools/call
TOOL_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_server_time","arguments":{}}}'
echo "$TOOL_REQUEST" | ./target/release/mcp-binance-server > "$TMP_OUT" 2> "$TMP_ERR" &
TOOL_PID=$!

sleep 2

if grep -q '"serverTime"' "$TMP_OUT"; then
    print_pass "serverTime field present in response"

    # Extract timestamp and validate it's reasonable (not checking exact time due to complexity)
    TIMESTAMP=$(grep -o '"serverTime":[0-9]*' "$TMP_OUT" | cut -d':' -f2)
    if [ ! -z "$TIMESTAMP" ] && [ "$TIMESTAMP" -gt 1000000000000 ]; then
        print_pass "Timestamp is valid format (>1000000000000)"
    else
        print_fail "Timestamp format invalid"
    fi
else
    print_fail "serverTime not found in response"
    cat "$TMP_OUT"
fi

# Cleanup
kill $SERVER_PID 2>/dev/null || true
kill $TOOL_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true
wait $TOOL_PID 2>/dev/null || true

# Scenario 3: Secure Credential Management
print_header "Scenario 3: Secure Credential Management"

print_info "Test A: With credentials (masking)"

export BINANCE_API_KEY="test_api_key_1234567890abcdef"
export BINANCE_SECRET_KEY="test_secret_key_abcdef1234567890"

./target/release/mcp-binance-server > "$TMP_OUT" 2> "$TMP_ERR" &
SERVER_PID=$!
sleep 2

if grep -q "API credentials configured" "$TMP_ERR"; then
    print_pass "Credential loading logged"

    if grep -q "test_...cdef" "$TMP_ERR"; then
        print_pass "API key properly masked (test_...cdef)"
    else
        print_fail "API key not masked correctly"
    fi

    if ! grep -q "test_secret_key_abcdef1234567890" "$TMP_ERR"; then
        print_pass "Secret key NOT exposed in logs"
    else
        print_fail "Secret key exposed in logs!"
    fi
else
    print_fail "Credential loading message not found"
fi

kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

print_info "Test B: Without credentials"

unset BINANCE_API_KEY BINANCE_SECRET_KEY

./target/release/mcp-binance-server > "$TMP_OUT" 2> "$TMP_ERR" &
SERVER_PID=$!
sleep 2

if grep -q "No API credentials configured" "$TMP_ERR"; then
    print_pass "Warning logged for missing credentials"
else
    print_fail "Missing credential warning not found"
fi

if ps -p $SERVER_PID > /dev/null; then
    print_pass "Server still runs without credentials"
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
else
    print_fail "Server crashed without credentials"
fi

# Cleanup temp files
rm "$TMP_OUT" "$TMP_ERR"

# Final Report
print_header "Test Summary"

TOTAL=$((PASS + FAIL))
echo "Total Tests: $TOTAL"
echo -e "${GREEN}Passed: $PASS${NC}"
if [ $FAIL -gt 0 ]; then
    echo -e "${RED}Failed: $FAIL${NC}"
fi

echo ""
if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}========================================="
    echo "✓ ALL TESTS PASSED"
    echo "=========================================${NC}"
    exit 0
else
    echo -e "${RED}========================================="
    echo "✗ SOME TESTS FAILED"
    echo "=========================================${NC}"
    exit 1
fi
