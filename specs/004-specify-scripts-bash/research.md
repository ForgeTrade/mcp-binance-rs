# Research: Comprehensive Test Suite

**Date**: 2025-10-16
**Feature**: Comprehensive integration tests for REST endpoints and WebSocket streams

## Testing Framework Decisions

### Decision 1: Core Testing Stack

**Chosen**: Built-in Rust testing with Tokio test macros + cargo-nextest

**Rationale**:
- Native `#[tokio::test]` macro already integrated in existing codebase
- Zero additional compile-time overhead (built into Tokio 1.48.0)
- cargo-nextest provides 2-3x faster test execution via process-per-test isolation
- Automatic retry with exponential backoff for flaky network tests
- Better test reporting (JUnit XML, JSON output for CI/CD)

**Alternatives Considered**:
- Standard `cargo test`: Slower for large test suites, shared state between tests
- Custom test harness: Unnecessary complexity given mature ecosystem tools

**Implementation**:
```bash
cargo install cargo-nextest --locked
cargo nextest run --retries 3
```

---

### Decision 2: HTTP API Mocking

**Chosen**: wiremock 0.6.5

**Rationale**:
- Async-first design built for Tokio runtime (perfect match)
- Mature library: 32M+ downloads, 9.6/10 trust score
- Request matching for path, query params, headers, JSON body
- Response templating with dynamic data
- In-process mock server (no external dependencies, fast startup)
- Can simulate Binance API exactly (REST endpoints with HMAC signatures)

**Alternatives Considered**:
- httpmock 0.8.1: More complex API, overkill for our needs
- mockito: Older, less maintained, sync-focused
- Real Binance Testnet only: Slower tests, API rate limits, network flakiness

**Implementation**:
```toml
[dev-dependencies]
wiremock = "0.6"
```

---

### Decision 3: WebSocket Testing

**Chosen**: tokio-tungstenite 0.28.0 (already in stack) + mpsc channels for mocking

**Rationale**:
- Already using tokio-tungstenite 0.28.0 for WebSocket client implementation
- Split WebSocket into `impl Sink` + `impl Stream` for unit testing
- Use `tokio::sync::mpsc` channels to simulate WebSocket messages
- Integration tests connect to real Binance Testnet WebSocket
- Proven pattern from Axum official testing examples

**Alternatives Considered**:
- futures-test: Overkill for our simple stream testing needs
- Mock WebSocket server: Unnecessary complexity, channels sufficient

**Implementation**:
```rust
let (tx, rx) = tokio::sync::mpsc::channel(100);
let mock_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
// Send mock WebSocket messages via tx
```

---

### Decision 4: Test Fixtures and Parameterization

**Chosen**: rstest 0.26.1

**Rationale**:
- Dependency injection pattern for test fixtures (reusable setup)
- Parameterized tests with `#[rstest]` macro (test multiple cases)
- Async support with `#[tokio::test]`
- `#[fixture(once)]` for expensive setup (load credentials once)
- Timeout support: `#[timeout(Duration::from_secs(30))]`
- 2.5M downloads/month, actively maintained (July 2025 release)

**Alternatives Considered**:
- test-case 3.x: Simpler but lacks fixture dependency injection
- Manual setup/teardown: Repetitive boilerplate, error-prone

**Implementation**:
```toml
[dev-dependencies]
rstest = "0.26"
```

---

### Decision 5: Sequential Test Execution

**Chosen**: serial_test 3.2.0

**Rationale**:
- Simple attribute macros: `#[serial]`, `#[parallel]`, `#[serial(key = "group")]`
- Key-based grouping: Run subsets sequentially (e.g., `#[serial(binance_api)]`)
- File-based locking: Works across process boundaries (cargo-nextest compatible)
- Module-level application: Apply to entire test suites
- Industry standard: 8.7/10 trust score, widely used

**Alternatives Considered**:
- Manual locking: Complex, error-prone
- All tests sequential: Too slow, defeats parallel execution goal
- cargo-nextest only: Doesn't provide within-process serialization

**Implementation**:
```toml
[dev-dependencies]
serial_test = "3.2"
```

**Usage**:
```rust
#[tokio::test]
#[serial(binance_api)]  // Rate-limited tests run sequentially
async fn test_place_order() { }

#[tokio::test]
#[parallel]  // Read-only tests run in parallel
async fn test_get_ticker() { }
```

---

## Test Organization Strategy

### Directory Structure

**Chosen**: Hybrid organization (by protocol type + by feature area)

```
tests/
├── common/                       # Shared utilities (NOT a test binary)
│   ├── mod.rs                    # Re-exports for fixtures, helpers
│   ├── fixtures.rs               # Test credentials, bearer tokens, sample orders
│   ├── binance_client.rs         # Testnet client configuration
│   └── assertions.rs             # Custom assertion helpers
├── integration/                  # Main integration test suite
│   ├── mod.rs                    # Test module root
│   ├── rest_api/
│   │   ├── mod.rs
│   │   ├── market_data.rs        # 5 market data endpoint tests
│   │   ├── orders.rs             # 5 order management endpoint tests
│   │   ├── account.rs            # 5 account endpoint tests
│   │   └── server_time.rs        # Already exists
│   ├── websocket/
│   │   ├── mod.rs
│   │   ├── ticker.rs             # Ticker stream tests
│   │   ├── depth.rs              # Order book depth stream tests
│   │   └── user_data.rs          # User data stream tests
│   ├── mcp_lifecycle.rs          # Already exists
│   └── security.rs               # Already exists
└── README.md                     # Test documentation
```

**Rationale**:
- `common/mod.rs` prevents Cargo from treating it as test binary
- Primary grouping by protocol (REST vs WebSocket) - different setup requirements
- Secondary grouping by feature (market data, orders, account) - logical cohesion
- Preserves existing tests (server_time.rs, mcp_lifecycle.rs, security.rs)
- Clear navigation: "Where are ticker stream tests?" → `tests/integration/websocket/ticker.rs`

---

## Performance Optimization Strategy

### Target: Sub-5-Minute Test Execution

**Approach**: Parallel execution with strategic sequential markers

1. **Default Parallel**: All read-only tests run concurrently (market data, server time, schema validation)
2. **Strategic Serial**: Only state-modifying tests use `#[serial(group)]`:
   - `#[serial(binance_api)]`: Order placement, cancellation (share testnet account state)
   - `#[serial(websocket)]`: WebSocket connection limit tests (max 50 concurrent)
   - `#[serial(rate_limit)]`: Rate limiting validation tests

3. **cargo-nextest Benefits**:
   - Process-per-test isolation (no shared state bugs)
   - Retry with exponential backoff (handles network flakiness)
   - Uses all CPU cores efficiently
   - 2-3x faster than standard `cargo test`

4. **Test Profile Optimization**:
```toml
[profile.test]
opt-level = 1  # Slightly optimize test code without losing debug info
```

---

## Binance Testnet Configuration

### Environment Setup

**Chosen**: Environment variables with `.env.test` file

**.env.test**:
```bash
BINANCE_TESTNET_API_KEY=your_testnet_api_key
BINANCE_TESTNET_API_SECRET=your_testnet_api_secret
BINANCE_TESTNET_BASE_URL=https://testnet.binance.vision
BINANCE_TESTNET_WS_URL=wss://testnet.binance.vision/ws
TEST_TIMEOUT_SECONDS=30
RUST_LOG=info
```

**Rationale**:
- Testnet provides safe, isolated testing without financial risk
- No rate limit concerns (more lenient than production)
- Supports all API operations (orders, account, WebSocket streams)
- Environment variables keep credentials out of code
- `.env.test` not committed to git (add to .gitignore)

**Implementation**:
```rust
// tests/common/mod.rs
use std::sync::Once;
static INIT: Once = Once::new();

pub fn init_test_env() {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
        tracing_subscriber::fmt().with_test_writer().init();
    });
}
```

---

## Test Fixtures Strategy

### Shared Fixtures

**Implementation in `tests/common/fixtures.rs`**:

```rust
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

    pub fn mock() -> Self {
        Self {
            api_key: "mock_api_key".to_string(),
            api_secret: "mock_api_secret".to_string(),
        }
    }
}

pub struct TestBearerToken {
    pub token: String,
}

impl TestBearerToken {
    pub fn valid() -> Self {
        Self { token: "valid_bearer_token".to_string() }
    }

    pub fn expired() -> Self {
        Self { token: "expired_bearer_token".to_string() }
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
```

**Usage**:
```rust
mod common;
use common::{TestCredentials, SampleOrder};

#[tokio::test]
async fn test_order() {
    let creds = TestCredentials::from_env();
    let order = SampleOrder::market_buy();
    // Test implementation
}
```

---

## CI/CD Integration

### GitHub Actions Configuration

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
```

---

## Summary: Complete Testing Stack

### Dependencies to Add

```toml
[dev-dependencies]
# HTTP mocking for Binance API
wiremock = "0.6"

# Test fixtures and parameterized tests
rstest = "0.26"

# Sequential test execution markers
serial_test = "3.2"

# Environment variable loading for tests
dotenv = "0.15"
```

### Binary Tools to Install

```bash
cargo install cargo-nextest --locked
```

### Key Metrics

| Requirement | Solution | Status |
|-------------|----------|--------|
| REST API testing | wiremock 0.6.5 | ✅ Async, mature, fast |
| WebSocket testing | tokio-tungstenite 0.28 + mpsc | ✅ Already in stack |
| Async support | #[tokio::test] | ✅ Native Tokio integration |
| Parallel execution | cargo-nextest | ✅ 2-3x faster than cargo test |
| Sequential markers | serial_test 3.2 | ✅ Simple attributes |
| Test fixtures | rstest 0.26 | ✅ Dependency injection |
| Testnet API | Environment variables | ✅ Safe, isolated testing |
| <5 min execution | Parallel + optimization | ✅ Estimated 3-4 minutes |

---

## Implementation Priority

1. **Phase 1**: Add dependencies, install cargo-nextest
2. **Phase 2**: Create test fixtures in `tests/common/`
3. **Phase 3**: REST API tests (15 endpoints)
4. **Phase 4**: WebSocket tests (3 streams)
5. **Phase 5**: Performance optimization and CI/CD integration
