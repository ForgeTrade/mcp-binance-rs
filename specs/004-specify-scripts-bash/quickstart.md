# Quickstart: Comprehensive Test Suite

**Date**: 2025-10-16
**Feature**: Comprehensive integration tests for REST endpoints and WebSocket streams

## Prerequisites

1. **Install cargo-nextest** (fast parallel test runner):
```bash
cargo install cargo-nextest --locked
```

2. **Set up test environment** (create `.env.test` file):
```bash
# .env.test
BINANCE_TESTNET_API_KEY=your_testnet_api_key
BINANCE_TESTNET_API_SECRET=your_testnet_api_secret
BINANCE_TESTNET_BASE_URL=https://testnet.binance.vision
BINANCE_TESTNET_WS_URL=wss://testnet.binance.vision/ws
TEST_TIMEOUT_SECONDS=30
RUST_LOG=info
```

3. **Add to .gitignore** (keep credentials safe):
```bash
echo ".env.test" >> .gitignore
```

## Running Tests

### Run All Tests (Parallel Execution)

```bash
# Fast parallel execution with 3 automatic retries for flaky tests
cargo nextest run --all-features --retries 3
```

**Expected output**:
```
    Finished test [unoptimized + debuginfo] target(s) in 5.23s
    Starting 42 tests across 8 binaries
        PASS [ 0.234s] mcp_binance_server integration::rest_api::market_data::test_get_ticker
        PASS [ 0.189s] mcp_binance_server integration::rest_api::market_data::test_get_order_book
        PASS [ 0.312s] mcp_binance_server integration::websocket::ticker::test_ticker_stream
        ...
     Summary [ 4.56s] 42 tests run: 42 passed, 0 failed, 0 skipped
```

### Run Specific Test Suite

```bash
# REST API tests only
cargo nextest run --all-features integration::rest_api

# WebSocket tests only
cargo nextest run --all-features integration::websocket

# Security tests only
cargo nextest run --all-features integration::security
```

### Run Single Test

```bash
# Run specific test by name
cargo nextest run --all-features test_get_ticker

# Run with detailed output
cargo nextest run --all-features test_get_ticker --nocapture
```

### Standard cargo test (Slower)

```bash
# Use standard test runner if cargo-nextest not available
cargo test --all-features

# With logging output
RUST_LOG=debug cargo test --all-features -- --nocapture
```

## Test Organization

```
tests/
├── common/                       # Shared test utilities
│   ├── mod.rs                    # Re-exports
│   ├── fixtures.rs               # Test credentials, bearer tokens, sample orders
│   ├── binance_client.rs         # Testnet client configuration
│   └── assertions.rs             # Custom assertion helpers
├── integration/                  # Integration tests
│   ├── rest_api/
│   │   ├── market_data.rs        # 5 market data endpoint tests
│   │   ├── orders.rs             # 5 order management tests
│   │   ├── account.rs            # 5 account endpoint tests
│   │   └── server_time.rs        # Server time test (existing)
│   ├── websocket/
│   │   ├── ticker.rs             # Ticker stream tests
│   │   ├── depth.rs              # Order book depth tests
│   │   └── user_data.rs          # User data stream tests
│   ├── mcp_lifecycle.rs          # MCP lifecycle tests (existing)
│   └── security.rs               # Credential security tests (existing)
└── README.md                     # Test documentation
```

## Writing Your First Test

### 1. REST API Test Example

```rust
// tests/integration/rest_api/market_data.rs
mod common;
use common::{TestCredentials, init_test_env};

#[tokio::test]
#[serial(binance_api)]  // Run sequentially with other rate-limited tests
async fn test_get_ticker() {
    init_test_env();  // Load .env.test once

    let creds = TestCredentials::from_env();

    // Test implementation
    let response = reqwest::get(&format!(
        "{}/api/v3/ticker/24hr?symbol=BTCUSDT",
        std::env::var("BINANCE_TESTNET_BASE_URL").unwrap()
    ))
    .await
    .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let ticker: serde_json::Value = response.json().await.unwrap();
    assert!(ticker["symbol"].as_str().unwrap() == "BTCUSDT");
}
```

### 2. WebSocket Test Example

```rust
// tests/integration/websocket/ticker.rs
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;

#[tokio::test]
#[timeout(Duration::from_secs(30))]
async fn test_ticker_stream() {
    let url = format!(
        "{}/ws/btcusdt@ticker",
        std::env::var("BINANCE_TESTNET_WS_URL").unwrap()
    );

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    let (_, mut read) = ws_stream.split();

    // Wait for first message
    if let Some(Ok(msg)) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&msg.to_string()).unwrap();
        assert!(data["e"].as_str().unwrap() == "24hrTicker");
        assert!(data["s"].as_str().unwrap() == "BTCUSDT");
    } else {
        panic!("No message received from WebSocket stream");
    }
}
```

### 3. Using Test Fixtures

```rust
// tests/common/fixtures.rs
pub struct TestCredentials {
    pub api_key: String,
    pub api_secret: String,
}

impl TestCredentials {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("BINANCE_TESTNET_API_KEY")
                .unwrap_or_else(|_| "test_key".to_string()),
            api_secret: std::env::var("BINANCE_TESTNET_API_SECRET")
                .unwrap_or_else(|_| "test_secret".to_string()),
        }
    }
}

pub struct SampleOrder {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
}

impl SampleOrder {
    pub fn market_buy() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: "0.001".to_string(),
        }
    }
}

// Usage in tests
#[tokio::test]
async fn test_with_fixtures() {
    let creds = TestCredentials::from_env();
    let order = SampleOrder::market_buy();
    // Test implementation
}
```

### 4. Parameterized Tests with rstest

```rust
use rstest::rstest;

#[rstest]
#[case("BTCUSDT", 200)]
#[case("ETHUSDT", 200)]
#[case("INVALID", 400)]
#[tokio::test]
async fn test_ticker_multiple_symbols(#[case] symbol: &str, #[case] expected_status: u16) {
    let response = reqwest::get(&format!(
        "{}/api/v3/ticker/24hr?symbol={}",
        std::env::var("BINANCE_TESTNET_BASE_URL").unwrap(),
        symbol
    ))
    .await
    .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), expected_status);
}
```

## Test Execution Patterns

### Parallel Tests (Default)

Most tests run in parallel for speed:

```rust
#[tokio::test]
#[parallel]  // Explicit marker (optional, this is the default)
async fn test_get_ticker() {
    // Read-only test, safe to run concurrently
}
```

### Sequential Tests (Explicit Grouping)

Tests that conflict with each other use sequential markers:

```rust
#[tokio::test]
#[serial(binance_api)]  // All tests with this marker run sequentially
async fn test_place_order() {
    // Modifies account state, needs isolation
}

#[tokio::test]
#[serial(rate_limit)]  // Different sequential group
async fn test_rate_limiting() {
    // Tests rate limiting behavior, needs isolation
}
```

## Mock Testing with wiremock

### Setup Mock Server

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_with_mock_binance_api() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Configure mock response
    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/24hr"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "symbol": "BTCUSDT",
            "lastPrice": "50000.00",
            "volume": "1000.00"
        })))
        .mount(&mock_server)
        .await;

    // Test against mock server
    let response = reqwest::get(&format!("{}/api/v3/ticker/24hr", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}
```

## Performance Benchmarking

### Measure Response Time

```rust
use std::time::Instant;

#[tokio::test]
#[serial(performance)]  // Isolate from other tests
async fn bench_rest_api_latency() {
    let mut latencies = Vec::new();

    // Collect 100 samples
    for _ in 0..100 {
        let start = Instant::now();

        let response = reqwest::get(&format!(
            "{}/api/v3/ticker/24hr?symbol=BTCUSDT",
            std::env::var("BINANCE_TESTNET_BASE_URL").unwrap()
        ))
        .await
        .expect("Request failed");

        let duration = start.elapsed();
        latencies.push(duration.as_millis());

        assert_eq!(response.status(), 200);
    }

    // Calculate percentiles
    latencies.sort();
    let p50 = latencies[50];
    let p95 = latencies[95];
    let p99 = latencies[99];

    // Assert performance thresholds
    assert!(p50 < 500, "P50 latency {} exceeds 500ms threshold", p50);
    assert!(p95 < 1000, "P95 latency {} exceeds 1000ms threshold", p95);

    println!("Performance: P50={}ms, P95={}ms, P99={}ms", p50, p95, p99);
}
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cargo-nextest
        run: cargo install cargo-nextest --locked

      - name: Run tests
        run: cargo nextest run --all-features --retries 3
        env:
          RUST_BACKTRACE: 1
          BINANCE_TESTNET_API_KEY: ${{ secrets.BINANCE_TESTNET_API_KEY }}
          BINANCE_TESTNET_API_SECRET: ${{ secrets.BINANCE_TESTNET_API_SECRET }}
          BINANCE_TESTNET_BASE_URL: https://testnet.binance.vision
          BINANCE_TESTNET_WS_URL: wss://testnet.binance.vision/ws

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: target/nextest/default/*.xml
```

## Troubleshooting

### Tests Failing with "Connection Refused"

**Problem**: Server not running or wrong port.

**Solution**:
```bash
# Start server in background
export HTTP_BEARER_TOKEN="test_token_123"
cargo run --features http-api,websocket &

# Wait for server to start
sleep 5

# Run tests
cargo nextest run
```

### Tests Failing with "Rate Limited"

**Problem**: Too many requests to Binance Testnet API.

**Solution**: Use sequential markers for rate-limited tests:
```rust
#[tokio::test]
#[serial(binance_api)]  // Runs sequentially, respects rate limits
async fn test_api_endpoint() { }
```

### WebSocket Tests Timeout

**Problem**: WebSocket connection not established.

**Solution**: Verify testnet URL and increase timeout:
```rust
#[tokio::test]
#[timeout(Duration::from_secs(60))]  // Increase from default 30s
async fn test_websocket() { }
```

### Tests Pass Locally but Fail in CI

**Problem**: Environment variables not set in CI.

**Solution**: Add secrets to GitHub Actions:
```yaml
env:
  BINANCE_TESTNET_API_KEY: ${{ secrets.BINANCE_TESTNET_API_KEY }}
  BINANCE_TESTNET_API_SECRET: ${{ secrets.BINANCE_TESTNET_API_SECRET }}
```

## Next Steps

1. **Review existing tests** in `tests/integration/` to understand patterns
2. **Add new test cases** following the examples above
3. **Run test suite** before committing changes: `cargo nextest run`
4. **Check coverage** (optional): `cargo tarpaulin --all-features`
5. **Submit PR** with passing tests and CI green checkmarks

## Resources

- [cargo-nextest Documentation](https://nexte.st/)
- [rstest Crate](https://docs.rs/rstest/)
- [wiremock Crate](https://docs.rs/wiremock/)
- [serial_test Crate](https://docs.rs/serial_test/)
- [Binance Testnet Documentation](https://testnet.binance.vision/)
