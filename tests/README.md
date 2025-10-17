# MCP Binance Server - Integration Tests

Comprehensive integration test suite for the MCP Binance Server, covering REST API endpoints, WebSocket streams, security, error handling, and performance.

## 📋 Test Coverage

### Phase 1-2: Infrastructure (12 tests)
- ✅ Test dependencies (wiremock, rstest, serial_test, dotenv)
- ✅ Test fixtures (credentials, bearer tokens, sample orders)
- ✅ JSON schema assertions
- ✅ HTTP client factories

### Phase 3: REST API Tests (15 tests)
**Market Data Endpoints (5 tests)**
- `test_ticker_24hr` - 24hr price statistics
- `test_order_book_depth` - Order book bids/asks
- `test_recent_trades` - Recent trades list
- `test_klines` - Candlestick/OHLCV data
- `test_average_price` - Current average price

**Order Management Endpoints (5 tests, sequential)**
- `test_place_order` - POST /api/v3/order
- `test_query_order` - GET /api/v3/order
- `test_cancel_order` - DELETE /api/v3/order
- `test_open_orders` - GET /api/v3/openOrders
- `test_all_orders` - GET /api/v3/allOrders

**Account Endpoints (5 tests)**
- `test_account_info` - Account balances
- `test_my_trades` - Trade history
- `test_rate_limit_info` - Rate limit status
- `test_start_user_data_stream` - Start user stream
- `test_close_user_data_stream` - Close user stream

### Phase 4: WebSocket Tests (6 tests)
- `test_ticker_websocket_stream` - Real-time ticker updates
- `test_depth_websocket_stream` - Order book updates
- `test_user_data_stream_subscription` - User data stream setup
- `test_user_data_stream_connection` - User stream connection
- `test_websocket_reconnection` - Reconnection handling
- `test_multiple_stream_subscriptions` - Multi-stream support

### Phase 5: Security Tests (4 tests)
- `test_bearer_token_validation` - Token validation
- `test_invalid_api_key_rejection` - Invalid key handling
- `test_mcp_authentication_flow` - MCP auth flow
- `test_credential_leak_prevention` - Credential protection

### Phase 6: Error Handling Tests (7 tests)
- `test_network_timeout_handling` - Timeout behavior
- `test_api_error_responses` - 400/429/500 handling
- `test_retry_logic_exponential_backoff` - Retry logic
- `test_invalid_request_parameters` - Invalid params
- `test_rate_limit_exceeded_handling` - Rate limit detection
- `test_connection_failure_recovery` - Connection recovery
- `test_malformed_json_handling` - Invalid JSON handling

### Phase 7: Performance Tests (5 tests)
- `test_response_time_benchmarks` - Response time measurement
- `test_concurrent_request_throughput` - Concurrent load testing
- `test_memory_usage_profiling` - Memory footprint
- `test_websocket_message_latency` - WebSocket latency
- `test_load_testing_stress` - Stress testing

## 🚀 Running Tests

### Prerequisites

1. **Copy environment template:**
   ```bash
   cp .env.test.example .env.test
   ```

2. **Get Binance Testnet credentials:**
   - Visit https://testnet.binance.vision/
   - Create an account and generate API keys
   - Update `.env.test` with your credentials

3. **Configure environment:**
   ```bash
   # Required
   BINANCE_TESTNET_API_KEY=your_testnet_api_key
   BINANCE_TESTNET_API_SECRET=your_testnet_api_secret

   # Optional (defaults provided)
   BINANCE_TESTNET_BASE_URL=https://testnet.binance.vision
   BINANCE_TESTNET_WS_URL=wss://stream.testnet.binance.vision
   HTTP_BEARER_TOKEN=test_token_123
   ```

### Run All Tests

```bash
# Using cargo test
cargo test --test integration

# Using cargo-nextest (faster, parallel execution)
cargo nextest run --test integration
```

### Run Specific Test Suites

```bash
# REST API tests only
cargo nextest run integration::rest_api

# WebSocket tests only
cargo nextest run integration::websocket

# Security tests only
cargo nextest run integration::security_extended

# Error handling tests
cargo nextest run integration::error_handling

# Performance tests
cargo nextest run integration::performance
```

### Run Specific Tests

```bash
# Market data tests
cargo nextest run test_ticker_24hr
cargo nextest run test_order_book_depth

# Order management tests (sequential)
cargo test --test integration test_place_order -- --test-threads=1

# WebSocket tests
cargo nextest run test_ticker_websocket_stream
```

## 📊 Test Organization

### Sequential vs Parallel Execution

**Parallel Tests** (can run concurrently):
- All market data endpoint tests
- All WebSocket stream tests (except user data)
- Security validation tests
- Error handling tests
- Performance benchmarks

**Sequential Tests** (must run one at a time):
- Order management tests - use `#[serial(orders)]`
- User data stream tests - use `#[serial(user_stream)]`
- MCP authentication tests - use `#[serial(mcp_auth)]`

### Test Structure

```
tests/
├── common/                    # Shared test utilities
│   ├── mod.rs                # Test initialization
│   ├── fixtures.rs           # Test data (credentials, orders)
│   ├── binance_client.rs     # HTTP client factories
│   └── assertions.rs         # JSON schema validators
├── integration/              # Integration tests
│   ├── mod.rs               # Test module root
│   ├── rest_api/            # REST API tests
│   │   ├── mod.rs           # Shared utilities
│   │   ├── market_data.rs   # Market data endpoints
│   │   ├── orders.rs        # Order management
│   │   └── account.rs       # Account endpoints
│   ├── websocket/           # WebSocket tests
│   │   ├── mod.rs           # WebSocket utilities
│   │   └── streams.rs       # Stream tests
│   ├── security_extended.rs # Security tests
│   ├── error_handling.rs    # Error tests
│   └── performance.rs       # Performance tests
└── README.md               # This file
```

## 🔧 Test Configuration

### Timeouts

- **HTTP requests**: 30 seconds (configurable in client)
- **WebSocket connections**: 30 seconds for first message
- **Performance tests**: 5 seconds per request

### Rate Limiting

Tests include automatic rate limit handling:
- 500ms delay between sequential requests
- Respect Binance Testnet rate limits (1200 requests/minute)

### Retry Logic

Error handling tests verify:
- Exponential backoff (100ms, 200ms, 400ms, ...)
- Maximum 3 retry attempts
- Graceful degradation on failures

## 📈 Success Criteria

- ✅ **REST API**: All 15 endpoints return correct responses
- ✅ **WebSocket**: All 3 streams deliver messages < 1 second
- ✅ **Security**: Credentials protected, auth validated
- ✅ **Errors**: Network failures handled gracefully
- ✅ **Performance**: Response times < 5 seconds, 90%+ success rate

## 🐛 Troubleshooting

### Common Issues

**1. Tests fail with "No such file or directory"**
```bash
# Solution: Ensure .env.test exists
cp .env.test.example .env.test
```

**2. Tests fail with "401 Unauthorized"**
```bash
# Solution: Update credentials in .env.test
# Get new keys from https://testnet.binance.vision/
```

**3. WebSocket tests timeout or 404 errors**
```bash
# Solution: Ensure correct WebSocket URL in .env.test
# Correct: BINANCE_TESTNET_WS_URL=wss://stream.testnet.binance.vision
# Wrong:   BINANCE_TESTNET_WS_URL=wss://testnet.binance.vision/ws
# The /ws/<stream> path is automatically added by the test helper
```

**4. Order tests fail due to insufficient balance**
```bash
# Solution: Use Binance Testnet faucet to get test funds
# Visit testnet.binance.vision and use faucet feature
```

## 📚 Additional Resources

- [Binance API Documentation](https://binance-docs.github.io/apidocs/spot/en/)
- [Binance Testnet](https://testnet.binance.vision/)
- [MCP Protocol Specification](https://modelcontextprotocol.io/docs)
- [cargo-nextest](https://nexte.st/)

## 🤝 Contributing

When adding new tests:
1. Follow existing test patterns and naming conventions
2. Add appropriate `#[serial(...)]` markers for sequential tests
3. Include CORS header validation for REST API tests
4. Update this README with test descriptions
5. Ensure tests pass both locally and in CI
