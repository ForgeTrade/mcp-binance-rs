# Implementation Tasks: Comprehensive Test Suite

**Feature**: Comprehensive integration tests for REST endpoints and WebSocket streams
**Branch**: `004-specify-scripts-bash`
**Total Tasks**: 58
**Est. Completion**: 3-4 hours

## Overview

This feature implements comprehensive integration tests for all 15 REST API endpoints and 3 WebSocket streams using Binance Testnet API. Tests execute in parallel by default (via cargo-nextest) with explicit sequential markers for conflicting tests. Target: sub-5-minute test execution with 80% test coverage of critical paths.

## Dependencies & Execution Order

```
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (US1: REST API) → Phase 4 (US2: WebSocket)
                                         ↓
                                  Phase 5 (US3: Security - extends existing)
                                         ↓
                                  Phase 6 (US4: Error Handling)
                                         ↓
                                  Phase 7 (US5: Performance)
                                         ↓
                                  Phase 8 (Polish & CI/CD)
```

**Independent User Stories**: US1, US2, US4, US5 can be tested independently
**Dependent**: US3 extends existing security.rs tests
**MVP Scope**: Phase 1 + Phase 2 + Phase 3 (US1: REST API tests)

---

## Phase 1: Setup & Dependencies

**Goal**: Configure test environment and install dependencies

- [X] T001 Add test dependencies to Cargo.toml ([dev-dependencies]: wiremock 0.6, rstest 0.26, serial_test 3.2, dotenv 0.15)
- [X] T002 Create .env.test.example file with required environment variables (BINANCE_TESTNET_API_KEY, BINANCE_TESTNET_API_SECRET, BINANCE_TESTNET_BASE_URL, BINANCE_TESTNET_WS_URL, TEST_TIMEOUT_SECONDS, RUST_LOG)
- [X] T003 Add .env.test to .gitignore to prevent credential leakage
- [X] T004 Install cargo-nextest binary tool via `cargo install cargo-nextest --locked` (global install, available for all subsequent test phases)
- [X] T005 Update Cargo.toml test profile for optimization ([profile.test] opt-level = 1)

**Acceptance**: Cargo.toml has all test dependencies, .env.test.example created, cargo-nextest installed

---

## Phase 2: Foundational Test Infrastructure

**Goal**: Create shared test utilities and fixtures

- [X] T006 Create tests/common/mod.rs with module re-exports and init_test_env() function
- [X] T007 [P] Create tests/common/fixtures.rs with TestCredentials struct and from_env()/mock() methods (note: bearer token = HTTP_BEARER_TOKEN env var)
- [X] T008 [P] Add TestBearerToken struct to tests/common/fixtures.rs with valid()/expired() methods
- [X] T009 [P] Add SampleOrder struct to tests/common/fixtures.rs with market_buy()/limit_sell() methods
- [X] T010 Create tests/common/binance_client.rs with testnet client configuration helper
- [X] T011 Create tests/common/assertions.rs with custom assertion helpers for JSON schema validation
- [X] T012 Create tests/integration/mod.rs as test module root

**Acceptance**: All common utilities compile, fixtures load from environment, init_test_env() runs without errors

---

## Phase 3: User Story 1 - REST API Integration Tests (Priority: P1)

**Goal**: Validate all 15 REST API endpoints (5 market data, 5 order management, 5 account)

**Independent Test Criteria**:
- REST API test suite runs in isolation: `cargo nextest run integration::rest_api`
- All 15 endpoints return correct responses matching Binance API contracts
- Authentication errors return 401, rate limits return 429, invalid params return 400

### Setup

- [X] T013 [US1] Create tests/integration/rest_api/mod.rs with shared REST API test utilities

### Market Data Endpoints (5 tests)

- [X] T014 [P] [US1] Test GET /ticker endpoint in tests/integration/rest_api/market_data.rs (24hr ticker for BTCUSDT)
- [X] T015 [P] [US1] Test GET /depth endpoint in tests/integration/rest_api/market_data.rs (order book depth for BTCUSDT)
- [X] T016 [P] [US1] Test GET /trades endpoint in tests/integration/rest_api/market_data.rs (recent trades for BTCUSDT)
- [X] T017 [P] [US1] Test GET /klines endpoint in tests/integration/rest_api/market_data.rs (candlestick data for BTCUSDT)
- [X] T018 [P] [US1] Test GET /avgPrice endpoint in tests/integration/rest_api/market_data.rs (average price for BTCUSDT)

### Order Management Endpoints (5 tests)

- [X] T019 [US1] Test POST /order endpoint in tests/integration/rest_api/orders.rs with #[serial(binance_api)] (place test order)
- [X] T020 [US1] Test GET /order endpoint in tests/integration/rest_api/orders.rs with #[serial(binance_api)] (query order status)
- [X] T021 [US1] Test DELETE /order endpoint in tests/integration/rest_api/orders.rs with #[serial(binance_api)] (cancel order)
- [X] T022 [US1] Test GET /openOrders endpoint in tests/integration/rest_api/orders.rs with #[serial(binance_api)] (list open orders)
- [X] T023 [US1] Test GET /allOrders endpoint in tests/integration/rest_api/orders.rs with #[serial(binance_api)] (order history)

### Account Endpoints (5 tests)

- [X] T024 [US1] Test GET /account endpoint in tests/integration/rest_api/account.rs with #[serial(binance_api)] (account information)
- [X] T025 [P] [US1] Test GET /myTrades endpoint in tests/integration/rest_api/account.rs (trade history for symbol)
- [X] T026 [P] [US1] Test GET /rateLimit/order endpoint in tests/integration/rest_api/account.rs (query rate limit status)
- [X] T027 [P] [US1] Test POST /userDataStream endpoint in tests/integration/rest_api/account.rs (start user data stream)
- [X] T028 [P] [US1] Test DELETE /userDataStream endpoint in tests/integration/rest_api/account.rs (close user data stream)
- [X] T028a [P] [US1] Verify CORS headers present in all REST API responses in tests/integration/rest_api/market_data.rs (check Access-Control-Allow-Origin, Access-Control-Allow-Methods)

**Phase 3 Success Criteria**:
- ✅ All 15 REST API endpoint tests pass with 100% success rate (SC-001)
- ✅ Authentication tests verify 401 responses for invalid tokens (SC-002)
- ✅ Tests complete in <2 minutes with parallel execution

---

## Phase 4: User Story 2 - WebSocket Integration Tests (Priority: P1)

**Goal**: Validate 3 WebSocket streams (ticker, depth, user data)

**Independent Test Criteria**:
- WebSocket test suite runs in isolation: `cargo nextest run integration::websocket`
- All streams deliver messages within 1 second and match expected JSON schema
- Connection failures trigger automatic reconnection within 30 seconds

### Setup

- [X] T029 [US2] Create tests/integration/websocket/mod.rs with shared WebSocket test utilities

### WebSocket Stream Tests (3 tests)

- [X] T030 [P] [US2] Test ticker stream in tests/integration/websocket/ticker.rs (BTCUSDT 24hr ticker updates)
- [X] T031 [P] [US2] Test depth stream in tests/integration/websocket/depth.rs (BTCUSDT order book updates with sequence numbers)
- [X] T032 [US2] Test user data stream in tests/integration/websocket/user_data.rs with #[serial(websocket)] (executionReport and balanceUpdate events)
- [X] T033 [US2] Test WebSocket reconnection logic in tests/integration/websocket/ticker.rs (simulate disconnect, verify reconnect <30s)
- [X] T034 [US2] Test WebSocket connection limit in tests/integration/websocket/ticker.rs with #[serial(websocket)] (verify max 50 concurrent connections)

**Phase 4 Success Criteria**:
- ✅ WebSocket message delivery latency <200ms for 50 concurrent connections (SC-003)
- ✅ Reconnection tests achieve 100% success rate (SC-008)
- ✅ Tests complete in <2 minutes

---

## Phase 5: User Story 3 - Authentication Security Tests (Priority: P1)

**Goal**: Extend existing security tests to cover bearer tokens and HMAC signatures

**Independent Test Criteria**:
- Security test suite runs in isolation: `cargo nextest run integration::security`
- Zero secrets appear in logs or error messages (SC-006)
- All unauthorized requests receive 401 responses

- [X] T035 [P] [US3] Add test for valid bearer token authentication in tests/integration/security.rs
- [X] T036 [P] [US3] Add test for expired bearer token rejection in tests/integration/security.rs
- [X] T037 [P] [US3] Add test for HMAC SHA256 signature generation in tests/integration/security.rs
- [X] T038 [P] [US3] Add test verifying no secrets in INFO/WARN logs in tests/integration/security.rs

**Phase 5 Success Criteria**:
- ✅ Security audit confirms zero secrets in logs (SC-006)
- ✅ 100% of unauthorized requests receive 401 responses (SC-002)

---

## Phase 6: User Story 4 - Error Handling Tests (Priority: P2)

**Goal**: Cover 20 distinct error scenarios (network failures, API errors, validation errors, timeouts)

**Independent Test Criteria**:
- Error handling test suite runs in isolation: `cargo nextest run integration::rest_api::error_handling`
- All error scenarios return appropriate HTTP status codes (400/429/503/504)
- Tests use wiremock to simulate Binance API errors

- [X] T039 [P] [US4] Test 503 Service Unavailable → 504 Gateway Timeout mapping in tests/integration/rest_api/error_handling.rs
- [X] T040 [P] [US4] Test network timeout (>10s) error handling in tests/integration/rest_api/error_handling.rs
- [X] T041 [P] [US4] Test malformed JSON → 400 Bad Request in tests/integration/rest_api/error_handling.rs
- [X] T042 [P] [US4] Test rate limit exceeded → 429 with Retry-After header in tests/integration/rest_api/error_handling.rs with #[serial(rate_limit)]
- [X] T043 [P] [US4] Test invalid parameters → 400 with validation errors in tests/integration/rest_api/error_handling.rs
- [X] T044 [P] [US4] Test missing authentication → 401 Unauthorized in tests/integration/rest_api/error_handling.rs
- [X] T045 [P] [US4] Test WebSocket malformed JSON error handling in tests/integration/websocket/error_handling.rs

**Phase 6 Success Criteria**:
- ✅ At least 20 distinct error scenarios covered (SC-004)
- ✅ All error tests verify proper status code mapping (FR-008)

---

## Phase 7: User Story 5 - Performance & Load Tests (Priority: P3)

**Goal**: Measure response times, throughput, memory usage under load

**Independent Test Criteria**:
- Performance test suite runs in isolation: `cargo nextest run integration::performance`
- Median REST API response time <500ms with 1000 concurrent requests (SC-005)
- Tests run sequentially with #[serial(performance)] to avoid interference

- [X] T046 [US5] Test REST API response time (median <500ms, 1000 requests) in tests/integration/performance/rest_api_latency.rs with #[serial(performance)]
- [X] T047 [US5] Test WebSocket message throughput (50 connections, <200ms latency) in tests/integration/performance/websocket_throughput.rs with #[serial(performance)]
- [X] T048 [US5] Test rate limiting accuracy (100 requests/minute) in tests/integration/performance/rate_limiting.rs with #[serial(performance)]
- [X] T049 [US5] Test WebSocket memory stability (1 hour connection) in tests/integration/performance/memory_leak.rs with #[serial(performance)]
- [X] T050 [US5] Test graceful shutdown (all connections close <10s) in tests/integration/performance/graceful_shutdown.rs with #[serial(performance)]

**Phase 7 Success Criteria**:
- ✅ Median response time <500ms under load (SC-005)
- ✅ WebSocket latency <200ms for 50 connections (SC-003)
- ✅ Rate limiting tests confirm 100% accuracy (SC-009)

---

## Phase 8: Polish & Cross-Cutting Concerns

**Goal**: Documentation, CI/CD integration, and final polish

- [X] T051 [P] Create tests/README.md with test execution guide and troubleshooting tips (outline: Prerequisites, Running Tests, Test Organization, Troubleshooting, CI/CD Integration - reference quickstart.md for content)
- [X] T052 [P] Add GitHub Actions workflow .github/workflows/tests.yml with cargo-nextest integration
- [X] T053 Verify test suite completes in <5 minutes on CI/CD hardware (SC-007)
- [X] T054 Run `cargo nextest run --all-features --retries 3` and verify all tests pass
- [X] T055 Verify test coverage reaches 80% of critical paths using `cargo tarpaulin` (SC-010)
- [X] T056 Update project README.md with test execution instructions
- [X] T057 Create .specify/docs/testing-strategy.md documenting test organization and patterns

**Phase 8 Success Criteria**:
- ✅ Test suite completes in <5 minutes (SC-007)
- ✅ 80% test coverage of critical paths (SC-010)
- ✅ CI/CD pipeline runs tests on every commit

---

## Parallel Execution Examples

### Phase 2 - Foundational (All Parallel)
```bash
# All fixture files can be created in parallel
cargo check tests/common/fixtures.rs &
cargo check tests/common/binance_client.rs &
cargo check tests/common/assertions.rs &
wait
```

### Phase 3 - REST API Tests

**Parallel Group 1** (Market Data - Read-only):
```bash
cargo nextest run test_get_ticker &
cargo nextest run test_get_depth &
cargo nextest run test_get_trades &
cargo nextest run test_get_klines &
cargo nextest run test_avg_price &
wait
```

**Sequential Group** (Order Management - Modifies account state):
```bash
cargo nextest run test_place_order
cargo nextest run test_query_order
cargo nextest run test_cancel_order
cargo nextest run test_open_orders
cargo nextest run test_all_orders
```

**Parallel Group 2** (Account - Read-only):
```bash
cargo nextest run test_account_info &
cargo nextest run test_my_trades &
cargo nextest run test_rate_limit &
wait
```

### Phase 4 - WebSocket Tests

**Parallel Group**:
```bash
cargo nextest run test_ticker_stream &
cargo nextest run test_depth_stream &
wait
```

**Sequential Group** (Connection limits):
```bash
cargo nextest run test_user_data_stream
cargo nextest run test_reconnection
cargo nextest run test_connection_limit
```

---

## Implementation Strategy

### MVP Scope (Immediate Value)
**Complete first**: Phase 1 + Phase 2 + Phase 3 (US1: REST API tests)
- **Why**: REST API tests provide immediate regression protection for 15 endpoints
- **Deliverable**: 15 passing REST API integration tests with parallel execution
- **Time**: ~2 hours

### Incremental Delivery
1. **Iteration 1**: Phase 1-3 (MVP - REST API tests) → Deploy to feature branch
2. **Iteration 2**: Phase 4 (WebSocket tests) → Merge WebSocket coverage
3. **Iteration 3**: Phase 5-6 (Security + Error handling) → Comprehensive error coverage
4. **Iteration 4**: Phase 7-8 (Performance + Polish) → Production-ready test suite

### Testing Each User Story Independently

**US1 (REST API)**:
```bash
cargo nextest run integration::rest_api --retries 3
```

**US2 (WebSocket)**:
```bash
cargo nextest run integration::websocket --retries 3
```

**US3 (Security)**:
```bash
cargo nextest run integration::security --retries 3
```

**US4 (Error Handling)**:
```bash
cargo nextest run integration::rest_api::error_handling --retries 3
cargo nextest run integration::websocket::error_handling --retries 3
```

**US5 (Performance)**:
```bash
cargo nextest run integration::performance --retries 3 --test-threads=1
```

---

## Task Summary

| Phase | User Story | Task Count | Parallel Tasks | Sequential Tasks | Est. Time |
|-------|-----------|------------|----------------|------------------|-----------|
| 1 | Setup | 5 | 3 | 2 | 15 min |
| 2 | Foundational | 7 | 7 | 0 | 20 min |
| 3 | US1 (REST API) | 17 | 12 | 5 | 60 min |
| 4 | US2 (WebSocket) | 6 | 2 | 4 | 30 min |
| 5 | US3 (Security) | 4 | 4 | 0 | 20 min |
| 6 | US4 (Error Handling) | 7 | 7 | 0 | 30 min |
| 7 | US5 (Performance) | 5 | 0 | 5 | 40 min |
| 8 | Polish | 7 | 5 | 2 | 25 min |
| **Total** | **5 User Stories** | **58** | **40** | **18** | **240 min (4h)** |

**Parallel Opportunities**: 69% of tasks can run in parallel (40/58)
**Critical Path**: Phase 1 → Phase 2 → Phase 3 → Phase 8 (MVP delivery)

---

## Success Criteria Validation

Each phase includes acceptance criteria mapped to success criteria from spec.md:

- **SC-001**: Phase 3 validates all 15 REST endpoints pass with 100% success rate
- **SC-002**: Phase 5 verifies 100% of unauthorized requests receive 401 responses
- **SC-003**: Phase 4 confirms WebSocket latency <200ms for 50 concurrent connections
- **SC-004**: Phase 6 covers 20+ distinct error scenarios
- **SC-005**: Phase 7 verifies median response time <500ms with 1000 concurrent requests
- **SC-006**: Phase 5 confirms zero secrets in logs
- **SC-007**: Phase 8 validates test suite completes in <5 minutes
- **SC-008**: Phase 4 achieves 100% reconnection success rate
- **SC-009**: Phase 7 confirms 100% rate limiting accuracy
- **SC-010**: Phase 8 reaches 80% test coverage of critical paths

---

## Dependencies Graph

```
T001-T005 (Setup)
    ↓
T006-T012 (Foundational)
    ↓
    ├─→ T013-T028 (US1: REST API) ─┐
    ├─→ T029-T034 (US2: WebSocket) ─┤
    ├─→ T035-T038 (US3: Security) ──┤
    ├─→ T039-T045 (US4: Error) ─────┤
    └─→ T046-T050 (US5: Performance) ┤
                                     ↓
                            T051-T057 (Polish)
```

**Blocking Dependencies**:
- Phase 2 (Foundational) blocks all user story phases
- All user story phases must complete before Phase 8 (Polish)

**No Internal Dependencies** (within each user story phase):
- US1, US2, US3, US4, US5 tests are independent and can be implemented in any order after Phase 2

---

## Notes

- **Test Execution**: Use `cargo nextest run --all-features --retries 3` for full test suite with automatic retry
- **Parallel Execution**: cargo-nextest automatically parallelizes tests by default
- **Sequential Markers**: Tests with `#[serial(group)]` run sequentially within their group
- **Environment**: All tests use Binance Testnet API exclusively (no production API usage)
- **Credentials**: Load from `.env.test` file (not committed to git)
- **CI/CD**: GitHub Actions runs tests on every commit with cargo-nextest
- **Performance**: Target <5 minutes total execution time via parallel execution strategy
