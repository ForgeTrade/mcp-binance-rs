# Data Model: Comprehensive Test Suite

**Date**: 2025-10-16
**Feature**: Comprehensive integration tests for REST endpoints and WebSocket streams

## Core Test Entities

### TestCase

Represents a single test scenario with given/when/then structure.

**Fields**:
- `test_id` (String): Unique identifier for the test (e.g., "rest_api_market_data_001")
- `test_name` (String): Descriptive name (e.g., "GET /ticker returns valid 24hr ticker data")
- `test_type` (Enum): REST_API | WEBSOCKET | AUTHENTICATION | ERROR_HANDLING | PERFORMANCE
- `priority` (Enum): P1 (Critical) | P2 (High) | P3 (Medium) | P4 (Low)
- `execution_mode` (Enum): PARALLEL | SEQUENTIAL
- `sequential_group` (Option<String>): Group key for sequential tests (e.g., "binance_api", "rate_limit")
- `timeout_seconds` (u32): Maximum execution time before failure (default: 30)
- `retry_count` (u8): Number of automatic retries for flaky tests (default: 0)
- `given` (String): Test preconditions
- `when` (String): Action being tested
- `then` (String): Expected outcome
- `fixtures_required` (Vec<String>): List of fixture IDs needed (e.g., ["test_credentials", "sample_order"])
- `status` (Enum): PENDING | RUNNING | PASSED | FAILED | SKIPPED

**Relationships**:
- Belongs to one `TestSuite`
- Uses zero or more `TestFixture` instances
- May produce one `TestResult`

**State Transitions**:
```
PENDING → RUNNING → {PASSED | FAILED | SKIPPED}
         ↓ (retry)
       RUNNING (if retry_count > 0)
```

**Validation Rules**:
- `test_id` must be unique within test suite
- `timeout_seconds` must be between 1 and 600 (10 minutes)
- `retry_count` must be between 0 and 5
- `sequential_group` required when `execution_mode` is SEQUENTIAL

---

### TestSuite

Collection of related test cases grouped by functionality.

**Fields**:
- `suite_id` (String): Unique identifier (e.g., "rest_api_market_data")
- `suite_name` (String): Descriptive name (e.g., "REST API Market Data Endpoints")
- `category` (Enum): REST_API | WEBSOCKET | SECURITY | PERFORMANCE | ERROR_HANDLING
- `test_count` (u32): Total number of tests in suite
- `setup_required` (bool): Whether suite needs setup/teardown
- `parallel_execution` (bool): Whether tests in suite can run in parallel (default: true)
- `environment` (Enum): TESTNET | MOCK | HYBRID

**Relationships**:
- Contains one or more `TestCase` instances
- Uses shared `TestFixture` instances across tests

**Validation Rules**:
- `suite_id` must be unique globally
- `test_count` must match actual number of TestCase instances
- All TestCase instances must have compatible `execution_mode` if `parallel_execution` is false

---

### TestFixture

Reusable test data and configuration shared across tests.

**Fields**:
- `fixture_id` (String): Unique identifier (e.g., "test_credentials", "sample_market_order")
- `fixture_type` (Enum): CREDENTIALS | BEARER_TOKEN | SAMPLE_ORDER | MOCK_RESPONSE | WEBSOCKET_MESSAGE
- `data` (JSON): Fixture content (structure varies by type)
- `scope` (Enum): GLOBAL | SUITE | TEST
- `lifecycle` (Enum): ONCE | PER_TEST | PER_SUITE
- `cleanup_required` (bool): Whether fixture needs teardown

**Fixture Type Schemas**:

**CREDENTIALS**:
```json
{
  "api_key": "string",
  "api_secret": "string",
  "environment": "TESTNET"
}
```

**BEARER_TOKEN**:
```json
{
  "token": "string",
  "expires_at": "ISO8601 timestamp",
  "valid": true
}
```

**SAMPLE_ORDER**:
```json
{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "order_type": "MARKET",
  "quantity": "0.001"
}
```

**MOCK_RESPONSE**:
```json
{
  "status_code": 200,
  "headers": {"Content-Type": "application/json"},
  "body": "{...}",
  "delay_ms": 0
}
```

**Relationships**:
- Used by one or more `TestCase` instances
- May reference other fixtures for composition

**Validation Rules**:
- `fixture_id` must be unique within scope
- `data` must match schema for `fixture_type`
- GLOBAL fixtures loaded once at test suite initialization
- PER_TEST fixtures created/destroyed for each test

---

### MockResponse

Simulated Binance API response for testing error scenarios.

**Fields**:
- `mock_id` (String): Unique identifier (e.g., "binance_rate_limit_error")
- `endpoint_pattern` (String): URL pattern to match (e.g., "/api/v3/order")
- `http_method` (Enum): GET | POST | PUT | DELETE
- `status_code` (u16): HTTP response code (200-599)
- `headers` (HashMap<String, String>): Response headers
- `body` (String): Response body (JSON or text)
- `delay_ms` (u64): Simulated network latency (default: 0)
- `match_conditions` (Vec<MatchCondition>): Request matching rules

**MatchCondition**:
- `match_type` (Enum): PATH | QUERY_PARAM | HEADER | BODY_JSON
- `key` (String): Parameter name (for query/header/json matches)
- `value` (String): Expected value
- `operator` (Enum): EQUALS | CONTAINS | REGEX

**Relationships**:
- Referenced by `TestCase` instances for mocking
- Stored in `TestFixture` with type MOCK_RESPONSE

**Validation Rules**:
- `status_code` must be valid HTTP code (100-599)
- `body` must be valid JSON when Content-Type is application/json
- `endpoint_pattern` must be valid URL pattern
- `delay_ms` should not exceed 10,000ms (10 seconds)

---

### PerformanceBenchmark

Measurable performance metric with pass/fail thresholds.

**Fields**:
- `benchmark_id` (String): Unique identifier (e.g., "rest_api_response_time_p50")
- `metric_name` (String): Human-readable name (e.g., "Median REST API Response Time")
- `metric_type` (Enum): LATENCY | THROUGHPUT | MEMORY_USAGE | CONNECTION_COUNT | ERROR_RATE
- `unit` (String): Measurement unit (e.g., "milliseconds", "requests/second", "MB")
- `threshold_value` (f64): Pass/fail boundary
- `threshold_operator` (Enum): LESS_THAN | LESS_THAN_EQUAL | GREATER_THAN | GREATER_THAN_EQUAL
- `actual_value` (Option<f64>): Measured value after test execution
- `percentile` (Option<u8>): For latency metrics (50, 95, 99)
- `sample_size` (u32): Number of measurements taken

**Relationships**:
- Associated with one or more `TestCase` instances of type PERFORMANCE
- Results stored in `TestResult`

**Validation Rules**:
- `threshold_value` must be positive
- `percentile` must be between 1 and 100 if specified
- `sample_size` must be at least 10 for statistical significance
- `actual_value` compared to `threshold_value` using `threshold_operator` to determine pass/fail

---

### TestResult

Output of a single test case execution.

**Fields**:
- `result_id` (String): Unique identifier
- `test_id` (String): Reference to TestCase
- `execution_timestamp` (ISO8601): When test started
- `duration_ms` (u64): Test execution time
- `status` (Enum): PASSED | FAILED | SKIPPED | ERROR
- `error_message` (Option<String>): Failure reason if status is FAILED or ERROR
- `stack_trace` (Option<String>): Full error trace for debugging
- `assertions_passed` (u32): Number of successful assertions
- `assertions_failed` (u32): Number of failed assertions
- `retry_attempt` (u8): Which retry iteration (0 = first attempt)
- `artifacts` (Vec<String>): URLs to logs, screenshots, or other debug data

**Relationships**:
- One-to-one with `TestCase` execution
- May reference `PerformanceBenchmark` results

**Validation Rules**:
- `execution_timestamp` must be valid ISO8601 format
- `status` must be FAILED or ERROR if `error_message` is present
- `assertions_passed` + `assertions_failed` must match total assertions in TestCase

---

## Entity Relationships

```
TestSuite
    │
    ├──(contains)──▶ TestCase [1..n]
    │                   │
    │                   ├──(uses)──▶ TestFixture [0..n]
    │                   │
    │                   ├──(produces)──▶ TestResult [0..1]
    │                   │
    │                   └──(measures)──▶ PerformanceBenchmark [0..n]
    │
    └──(shares)──▶ TestFixture [0..n]

TestFixture
    │
    └──(references)──▶ MockResponse [0..1]
        (when fixture_type = MOCK_RESPONSE)
```

---

## Test Execution Flow

```
1. Load TestSuite(s) from test configuration
2. Initialize GLOBAL TestFixtures (credentials, mock server)
3. For each TestSuite:
   a. Setup suite-level TestFixtures
   b. Determine execution order (parallel vs sequential groups)
   c. For each TestCase:
      i.   Load required TestFixtures
      ii.  Execute test (with retries if configured)
      iii. Collect TestResult
      iv.  Cleanup per-test fixtures
   d. Teardown suite-level fixtures
4. Aggregate TestResults into test report
5. Cleanup GLOBAL fixtures
```

---

## Example Test Configuration

```json
{
  "test_suite": {
    "suite_id": "rest_api_market_data",
    "suite_name": "REST API Market Data Endpoints",
    "category": "REST_API",
    "parallel_execution": true,
    "environment": "TESTNET",
    "test_cases": [
      {
        "test_id": "rest_api_ticker_001",
        "test_name": "GET /ticker returns valid 24hr ticker for BTCUSDT",
        "test_type": "REST_API",
        "priority": "P1",
        "execution_mode": "PARALLEL",
        "timeout_seconds": 30,
        "given": "Binance Testnet API is available and server is running",
        "when": "Client requests GET /ticker?symbol=BTCUSDT",
        "then": "Response is 200 OK with valid ticker JSON matching schema",
        "fixtures_required": ["test_credentials", "mock_binance_ticker_response"]
      }
    ]
  },
  "fixtures": [
    {
      "fixture_id": "test_credentials",
      "fixture_type": "CREDENTIALS",
      "scope": "GLOBAL",
      "lifecycle": "ONCE",
      "data": {
        "api_key": "testnet_api_key_from_env",
        "api_secret": "testnet_api_secret_from_env",
        "environment": "TESTNET"
      }
    }
  ]
}
```

---

## Design Decisions

### Why Hybrid Execution Model (Parallel + Sequential)?

**Problem**: Need fast test execution (<5 minutes) but some tests conflict:
- Rate limiting tests interfere with each other
- Connection limit tests need isolation
- Performance benchmarks require stable environment

**Solution**: Default to parallel execution, explicit sequential markers for conflicting tests:
```rust
#[tokio::test]
#[serial(binance_api)]  // Sequential group
async fn test_rate_limiting() { }

#[tokio::test]
#[parallel]  // Explicit parallel marker
async fn test_get_ticker() { }
```

**Benefit**: Achieves 2-3x faster execution while maintaining test reliability.

---

### Why TestFixture Lifecycle Management?

**Problem**: Tests need shared state (credentials, mock server) but also test isolation.

**Solution**: Three lifecycle scopes:
- **GLOBAL**: Load once (credentials, environment config) - shared across all tests
- **PER_SUITE**: Setup/teardown per test suite (mock server, database connection)
- **PER_TEST**: Fresh data per test (sample orders, bearer tokens) - prevents test pollution

**Benefit**: Balance between test speed (minimize setup/teardown) and isolation (prevent shared state bugs).

---

### Why MockResponse Entity?

**Problem**: Can't test error scenarios reliably by forcing real API failures.

**Solution**: Dedicated MockResponse entity with:
- Pattern matching for flexible request interception
- Simulated latency for timeout testing
- Composable conditions for complex scenarios

**Benefit**: Deterministic error testing without relying on flaky network conditions or rate limiting real API.
