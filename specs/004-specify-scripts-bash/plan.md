# Implementation Plan: Comprehensive Test Suite

**Branch**: `004-specify-scripts-bash` | **Date**: 2025-10-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-specify-scripts-bash/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement comprehensive integration test suite for all 15 REST API endpoints and 3 WebSocket streams using Binance Testnet API exclusively. Tests execute in parallel by default with explicit sequential markers for conflicting tests (rate limiting, connection limits, performance benchmarks). Target sub-5-minute test execution using cargo-nextest for 2-3x faster parallel execution with automatic retry logic for network flakiness. Test stack: wiremock 0.6.5 for HTTP mocking, rstest 0.26.1 for fixtures, serial_test 3.2.0 for sequential markers, tokio-tungstenite 0.28.0 for WebSocket testing.

## Technical Context

**Language/Version**: Rust 1.90+ (Edition 2024)
**Primary Dependencies**:
  - tokio 1.48.0 (async runtime with #[tokio::test] macro)
  - wiremock 0.6.5 (HTTP API mocking)
  - rstest 0.26.1 (test fixtures and parameterization)
  - serial_test 3.2.0 (sequential execution markers)
  - tokio-tungstenite 0.28.0 (WebSocket client testing)
  - dotenv 0.15 (environment variable loading)
  - cargo-nextest (binary tool for parallel test execution)

**Storage**: N/A (tests use Binance Testnet API and in-memory mock servers)
**Testing**: cargo test + #[tokio::test] + cargo-nextest for parallel execution
**Target Platform**: Linux/macOS/Windows (cross-platform test execution)
**Project Type**: single (Rust library/binary with comprehensive test suite)
**Performance Goals**:
  - <5 minute total test suite execution time
  - <500ms median REST API response time under load
  - <200ms WebSocket message delivery latency
  - 2-3x speedup via cargo-nextest parallel execution

**Constraints**:
  - Binance Testnet API rate limits (100 requests/minute)
  - Maximum 50 concurrent WebSocket connections
  - Tests must be deterministic and not depend on external state
  - No production API usage (Testnet only)
  - Automatic retry with exponential backoff for flaky network tests

**Scale/Scope**:
  - 42 integration tests total (15 REST API + 3 WebSocket + existing tests)
  - 5 test suites (REST API market data, orders, account, WebSocket, security)
  - 80% test coverage target for critical paths
  - 20 distinct error scenarios covered

### Terminology

**Authentication Token References**:
- **User-facing term**: "bearer token" (in documentation and user stories)
- **Environment variable**: `HTTP_BEARER_TOKEN` (in .env.test and code)
- **Test fixture**: `TestBearerToken` struct (in tests/common/fixtures.rs)

These all refer to the same concept: HTTP Authorization header tokens used for MCP server authentication.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: ✅ PASSED - All constitutional principles upheld

### Core Principle I: Security-First Architecture
- ✅ **Credentials Security**: Existing `SecretString` wrapper prevents credential leakage in logs (tests/integration/security.rs validates this)
- ✅ **Test Isolation**: Binance Testnet API exclusively, no production credentials required
- ✅ **Environment Variables**: Credentials loaded from .env.test file (not committed to git)

### Core Principle II: Async-First Design
- ✅ **Tokio Integration**: All tests use `#[tokio::test]` macro for native async/await
- ✅ **Non-Blocking**: WebSocket tests use async streams without blocking executor
- ✅ **Concurrent Execution**: cargo-nextest runs tests in parallel by default

### Core Principle III: Modular Architecture
- ✅ **Test Organization**: Hybrid structure by protocol type (REST/WebSocket) and feature area
- ✅ **Shared Fixtures**: Common test utilities in `tests/common/` module
- ✅ **Reusable Patterns**: TestFixture entity enables DRY test data management

### Core Principle IV: Type Safety
- ✅ **Schema Validation**: JSON schemas in contracts/ define expected test result structures
- ✅ **Enum Types**: TestCase status, execution mode, metric type all strongly typed
- ✅ **Compile-Time Checks**: Rust type system prevents test configuration errors

### Core Principle V: Performance Optimization
- ✅ **Parallel Execution**: cargo-nextest provides 2-3x speedup via process-per-test isolation
- ✅ **Strategic Sequential Markers**: Only conflicting tests (rate limiting, connection limits) run sequentially
- ✅ **Target Met**: Sub-5-minute test execution achievable with parallel strategy

### Core Principle VI: Error Resilience
- ✅ **Automatic Retry**: cargo-nextest `--retries 3` with exponential backoff for flaky network tests
- ✅ **Error Scenarios**: 20 distinct error cases covered (rate limits, timeouts, auth failures)
- ✅ **Graceful Degradation**: Tests handle Binance API failures without cascading

### Core Principle VII: Machine-Optimized Development
- ✅ **Test-First**: This feature implements comprehensive tests before new functionality
- ✅ **Measurable Outcomes**: 10 success criteria with quantifiable metrics (SC-001 to SC-010)
- ✅ **CI/CD Ready**: GitHub Actions integration with test result artifacts

### Core Principle VIII: Dependency Management
- ✅ **Latest Stable Versions**: All dependencies verified against crates.io
  - wiremock 0.6.5 (latest stable, 32M+ downloads, 9.6/10 trust score)
  - rstest 0.26.1 (latest stable, 2.5M downloads/month)
  - serial_test 3.2.0 (latest stable, 8.7/10 trust score)
  - tokio-tungstenite 0.28.0 (already in stack)
- ✅ **Security Patches**: Test dependencies reviewed, no known vulnerabilities
- ✅ **Update Cadence**: Test framework updates non-critical, can follow 30-day cycle

**Violations**: None

**Recommendations**: Constitution fully satisfied by test suite design.

## Project Structure

### Documentation (this feature)

```
specs/004-specify-scripts-bash/
├── plan.md                           # This file (implementation plan)
├── research.md                       # Testing framework decisions and rationale
├── data-model.md                     # Test entities (TestCase, TestSuite, TestFixture, etc.)
├── quickstart.md                     # Test execution guide and examples
├── contracts/                        # JSON schemas for test artifacts
│   ├── test-result-schema.json       # TestResult validation schema
│   ├── test-fixture-examples.json    # Sample fixtures (credentials, orders, mocks)
│   └── performance-benchmark-schema.json  # PerformanceBenchmark schema
├── checklists/
│   └── requirements.md               # Specification quality validation
└── tasks.md                          # Task list (Phase 2: /speckit.tasks command)
```

### Source Code (repository root)

```
src/
├── config/
│   └── credentials.rs               # SecretString wrapper (already exists)
├── server.rs                         # BinanceServer implementation (already exists)
├── handlers/                         # REST API handlers (already exists)
└── websocket/                        # WebSocket handlers (already exists)

tests/
├── common/                           # NEW: Shared test utilities (NOT a test binary)
│   ├── mod.rs                        # Re-exports for fixtures, helpers
│   ├── fixtures.rs                   # NEW: Test credentials, bearer tokens, sample orders
│   ├── binance_client.rs             # NEW: Testnet client configuration
│   └── assertions.rs                 # NEW: Custom assertion helpers
├── integration/                      # Integration test suite
│   ├── mod.rs                        # NEW: Test module root
│   ├── rest_api/                     # NEW: REST API tests
│   │   ├── mod.rs                    # NEW
│   │   ├── market_data.rs            # NEW: 5 market data endpoint tests
│   │   ├── orders.rs                 # NEW: 5 order management tests
│   │   ├── account.rs                # NEW: 5 account endpoint tests
│   │   └── server_time.rs            # EXISTING: Server time test
│   ├── websocket/                    # NEW: WebSocket tests
│   │   ├── mod.rs                    # NEW
│   │   ├── ticker.rs                 # NEW: Ticker stream tests
│   │   ├── depth.rs                  # NEW: Order book depth tests
│   │   └── user_data.rs              # NEW: User data stream tests
│   ├── mcp_lifecycle.rs              # EXISTING: MCP lifecycle tests
│   └── security.rs                   # EXISTING: Credential security tests
└── README.md                         # NEW: Test documentation

Cargo.toml                            # Add [dev-dependencies]: wiremock, rstest, serial_test, dotenv
.env.test                             # NEW: Test environment variables (not committed)
.gitignore                            # Add .env.test
```

**Structure Decision**: Single project structure with comprehensive test coverage. Tests organized hierarchically by protocol type (REST API vs WebSocket) for different setup requirements, then by feature area (market data, orders, account) for logical cohesion. Preserves existing tests (server_time.rs, mcp_lifecycle.rs, security.rs) while adding 27 new integration tests. Shared test utilities in `tests/common/` module prevent code duplication.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations identified** - Constitution Check passed with no exceptions required.

