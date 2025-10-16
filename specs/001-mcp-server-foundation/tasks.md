# Implementation Tasks: MCP Server Foundation

**Feature**: MCP Server Foundation
**Branch**: `001-mcp-server-foundation`
**Created**: 2025-10-16
**Status**: Ready for Implementation

## Task Format
`- [ ] T### [P#] [US#] Description (file: path/to/file.rs)`

Where:
- `T###` = Task ID (unique per feature)
- `P#` = Priority (P1/P2/P3 from spec)
- `US#` = User Story ID (US1/US2/US3, or none for cross-cutting)
- File path = Primary file to modify

---

## Phase 0: Project Setup & Infrastructure

### T001 - Repository Initialization
- [X] T001 [P1] [] Initialize Cargo workspace structure (file: Cargo.toml)
  - Create root Cargo.toml with workspace configuration
  - Set edition = "2021", Rust 1.75+ compatibility
  - Configure workspace members for future modularization
  - Add metadata: name, version (0.1.0), authors, license

- [X] T002 [P1] [] Add core dependencies to Cargo.toml (file: Cargo.toml)
  - Add rmcp 0.8 with features = ["server", "macros"]
  - Add tokio 1.x with features = ["full"]
  - Add reqwest 0.12 with features = ["json", "rustls-tls"]
  - Add serde 1.0 with features = ["derive"]
  - Add serde_json 1.0
  - Add schemars 1.0 with features = ["chrono04"]
  - Add thiserror 2
  - Add tracing 0.1
  - Add tracing-subscriber 0.3 with features = ["env-filter"]
  - Configure [profile.release] with lto=true, codegen-units=1, strip=true

- [X] T003 [P1] [] Create project directory structure (file: src/lib.rs)
  - Create src/ with lib.rs and main.rs
  - Create src/server/ module directory
  - Create src/tools/ module directory
  - Create src/binance/ module directory
  - Create src/config/ module directory
  - Create src/error.rs for error types
  - Create tests/integration/ directory
  - Create docs/ directory

- [X] T004 [P1] [] Configure .gitignore and documentation (file: .gitignore)
  - Verify reference directories excluded (already done)
  - Add Cargo build artifacts (/target, Cargo.lock for apps)
  - Add IDE files (.vscode/, .idea/, *.swp)
  - Create README.md with project overview
  - Create docs/DEVELOPMENT.md with build instructions
  - Setup pre-commit hook for cargo clippy, cargo fmt --check, cargo audit

---

## Phase 1: Error Handling & Type Foundations (Blocking)

### T005 - Error Infrastructure
- [X] T005 [P1] [] Define core error types with thiserror (file: src/error.rs)
  - Create McpError enum with variants:
    - ConnectionError(String) - Network failures
    - RateLimitError(String) - HTTP 429 responses
    - ParseError(String) - JSON deserialization failures
    - InvalidRequest(String) - MCP protocol violations
    - NotReady(String) - Server not initialized
    - InternalError(String) - Unexpected failures
  - Derive Error, Debug for McpError
  - Implement Display with user-friendly messages
  - Add error context metadata fields (optional)

- [X] T006 [P1] [] Implement error conversions (file: src/error.rs)
  - Impl From<reqwest::Error> for McpError
  - Impl From<serde_json::Error> for McpError
  - Impl From<std::io::Error> for McpError
  - Add helper methods: is_retryable(), error_type()
  - Ensure NO sensitive data in error messages

- [X] T007 [P1] [US3] Create SecretString type for credentials (file: src/config/credentials.rs)
  - Define SecretString newtype wrapper around String
  - Impl Debug with masking: SecretString(***)
  - Impl Display with truncation: {first4}...{last4}
  - NEVER derive Serialize
  - Add validation methods

---

## Phase 2: Configuration & Credential Management (US3)

### T008 - Environment Variable Loading
- [X] T008 [P1] [US3] Implement credential loading from env vars (file: src/config/credentials.rs)
  - Create Credentials struct with api_key, secret_key fields
  - Create from_env() method reading BINANCE_API_KEY, BINANCE_SECRET_KEY
  - Trim whitespace from values
  - Validate non-empty
  - Return Result with clear error messages
  - NO .env file loading

- [X] T009 [P1] [US3] Implement secure credential logging (file: src/config/credentials.rs)
  - Add mask_key() function: first4...last4 format
  - Log "API credentials configured (key: xxx...yyy)" at INFO level
  - Ensure secret_key NEVER logged at INFO/WARN levels
  - Add DEBUG-only full credential logging with [SENSITIVE DATA] prefix

- [X] T010 [P1] [US3] Handle missing credentials gracefully (file: src/config/credentials.rs)
  - Make credentials Option<Credentials> in server state
  - Log WARN "No API credentials configured; authenticated features disabled"
  - Allow server startup without credentials
  - Add is_authenticated() method to server

---

## Phase 3: Binance API Client (US2)

### T011 - HTTP Client Setup
- [X] T011 [P1] [US2] Create Binance HTTP client wrapper (file: src/binance/client.rs)
  - Create BinanceClient struct with reqwest::Client
  - Add base_url field (default: https://api.binance.com)
  - Configure timeout: 10 seconds default
  - Impl new() and with_timeout() constructors
  - Add user-agent header

- [X] T012 [P1] [US2] Define ServerTimeResponse type (file: src/binance/types.rs)
  - Create ServerTimeResponse struct
  - Add server_time: i64 field (milliseconds)
  - Derive Debug, Deserialize
  - Use #[serde(rename = "serverTime")] attribute
  - Add validation: must be positive

- [X] T013 [P1] [US2] Implement get_server_time API call (file: src/binance/client.rs)
  - Add async get_server_time(&self) -> Result<i64, McpError>
  - Construct URL: {base_url}/api/v3/time
  - Send GET request with timeout
  - Handle HTTP errors: 429 → RateLimitError, 5xx → ConnectionError
  - Parse JSON response to ServerTimeResponse
  - Return server_time value

- [X] T014 [P1] [US2] Implement exponential backoff for rate limits (file: src/binance/client.rs)
  - Add retry logic for 429 responses
  - Parse Retry-After header if present
  - Implement exponential backoff: 1s, 2s, 4s, 8s
  - Max 3 retries before failure
  - Log retry attempts at WARN level

---

## Phase 4: MCP Server Skeleton (US1)

### T015 - Server Infrastructure
- [X] T015 [P1] [US1] Create BinanceServer struct (file: src/server/mod.rs)
  - Define BinanceServer struct
  - Add fields: http_client, credentials (Option), ToolRouter
  - Impl new() constructor
  - Add ServerInfo configuration method
  - Ensure Clone trait for Arc sharing

- [X] T016 [P1] [US1] Implement ServerHandler trait (file: src/server/handler.rs)
  - Annotate impl with #[tool_handler]
  - Implement get_info() returning ServerInfo
  - Set protocol_version: ProtocolVersion::V_2024_11_05
  - Set capabilities: tools with list_changed=false
  - Set server_info: name="mcp-binance-server", version="0.1.0"
  - Add instructions: "Binance MCP Server for trading and market data"

- [X] T017 [P1] [US1] Setup stdio transport (file: src/main.rs)
  - Add #[tokio::main] to main function
  - Initialize tracing_subscriber with env filter
  - Configure logs to stderr with no ANSI colors
  - Create BinanceServer instance
  - Call service.serve(stdio()).await
  - Call server.waiting().await for blocking

- [X] T018 [P1] [US1] Implement graceful shutdown (file: src/main.rs)
  - Detect stdio closure
  - Log "MCP server shutting down" at INFO level
  - Cleanup resources if needed
  - Exit cleanly with status 0

---

## Phase 5: Tool Implementation (US1, US2)

### T019 - get_server_time Tool
- [X] T019 [P1] [US1] Create ToolRouter with macros (file: src/server/tool_router.rs)
  - Annotate impl with #[tool_router]
  - Prepare for tool method definitions
  - Ensure automatic JSON Schema generation

- [X] T020 [P1] [US2] Implement get_server_time tool (file: src/tools/get_server_time.rs)
  - Annotate with #[tool(description = "Returns current Binance server time...")]
  - Define async fn get_server_time(&self) -> Result<CallToolResult, McpError>
  - No parameters (use Parameters<EmptyParams> or no params)
  - Call self.client.get_server_time().await
  - On success: wrap in CallToolResult::success with Content::text(json)
  - On error: convert McpError with isError=true
  - JSON format: {"serverTime": 1234567890}

- [X] T021 [P1] [US2] Add time synchronization logging (file: src/tools/get_server_time.rs)
  - Calculate offset: server_time - local_time
  - Log at INFO: "Binance server time: {timestamp} (offset: {offset}ms)"
  - Warn if offset > 5 seconds

- [X] T022 [P1] [US1] Register tool in ToolRouter (file: src/server/tool_router.rs)
  - Ensure get_server_time is in #[tool_router] impl block
  - Verify tool appears in tools/list response
  - Verify JSON Schema generated correctly

---

## Phase 6: Integration & Testing

### T023 - MCP Protocol Testing
- [X] T023 [P1] [US1] Write MCP lifecycle integration test (file: tests/integration/mcp_lifecycle.rs)
  - Test: Send initialize request, verify response
  - Verify protocol_version = "2024-11-05"
  - Verify capabilities.tools present
  - Verify serverInfo.name = "mcp-binance-server"
  - Test: Send tools/list, verify get_server_time present
  - Test: Verify JSON Schema valid

- [X] T024 [P1] [US1] Test initialization without credentials (file: tests/integration/mcp_lifecycle.rs)
  - Unset BINANCE_API_KEY and BINANCE_SECRET_KEY
  - Start server, verify success
  - Check logs for WARN message
  - Verify server responds to initialize

- [X] T025 [P1] [US2] Write get_server_time tool test (file: tests/integration/server_time.rs)
  - Test: Call get_server_time, verify success response
  - Verify serverTime is valid i64 timestamp
  - Verify time within ±60s of local time
  - Test: Mock network failure, verify error response
  - Test: Mock 429 rate limit, verify retry behavior

- [X] T026 [P1] [US3] Test credential security (file: tests/integration/security.rs)
  - Test: Set credentials, start server
  - Capture INFO logs, verify NO full API key
  - Verify only masked format: xxxx...yyyy
  - Test: Trigger error, verify no credentials in error message
  - Test: Enable DEBUG logs, verify [SENSITIVE DATA] prefix present

---

## Phase 7: Documentation & Polish

### T027 - User-Facing Documentation
- [X] T027 [P1] [] Write README.md (file: README.md)
  - Add project description and features
  - Add prerequisites: Rust 1.90+, internet connection
  - Add installation: cargo build --release
  - Add configuration: environment variables
  - Add usage: Claude Desktop example config
  - Add quickstart command examples
  - Add troubleshooting section

- [X] T028 [P1] [] Create quickstart validation script (file: scripts/test_quickstart.sh)
  - Automate Scenario 1: MCP initialization
  - Automate Scenario 2: get_server_time call
  - Automate Scenario 3: credential loading tests
  - Output pass/fail for each success criteria

- [X] T029 [P2] [] Add inline code documentation (file: src/**/*.rs)
  - Add module-level doc comments
  - Document public APIs with /// comments
  - Add examples for non-obvious functions
  - Focus on "why" not "what"

- [X] T030 [P2] [] Create Claude Desktop setup guide (file: docs/CLAUDE_DESKTOP_SETUP.md)
  - Document claude_desktop_config.json format
  - Provide example configuration
  - Add troubleshooting steps
  - Document env var setup

---

## Phase 8: Performance & Validation

### T031 - Performance Validation
- [ ] T031 [P2] [] Benchmark MCP initialization (file: tests/benchmarks/init.rs)
  - Measure time from process start to `initialized` notification completion
  - Includes full handshake: initialize request + response + initialized notification + tools/list
  - Target: < 500ms total handshake time (SC-001)
  - Separately measure: (a) startup time to ready state, (b) handshake round-trip time
  - Log results to stderr

- [ ] T032 [P2] [] Benchmark get_server_time latency (file: tests/benchmarks/tool_latency.rs)
  - Measure tool call round-trip time
  - Target: < 100ms (SC-002)
  - Run 100 sequential calls
  - Check for memory leaks (SC-003)

- [ ] T033 [P2] [] Profile memory usage (file: tests/benchmarks/memory.rs)
  - Start server, measure idle memory
  - Target: < 50MB (from plan.md)
  - Execute 100 tool calls, verify no growth

---

## Phase 9: Edge Cases & Error Scenarios

### T034 - Edge Case Handling
- [ ] T034 [P2] [] Test rate limit handling (file: tests/integration/rate_limits.rs)
  - Spam requests to trigger 429
  - Verify exponential backoff applied
  - Verify error message includes retry-after
  - Verify no IP ban (418)

- [ ] T035 [P2] [] Test invalid JSON-RPC requests (file: tests/integration/error_handling.rs)
  - Send malformed JSON, verify parse error
  - Send invalid method, verify method_not_found
  - Send tool call before initialize, verify not_ready
  - Verify all errors are JSON-RPC 2.0 compliant

- [ ] T036 [P2] [] Test stdio disconnect handling (file: tests/integration/shutdown.rs)
  - Close stdin while server running
  - Verify graceful shutdown
  - Verify log message "MCP server shutting down"
  - Verify exit code 0

- [ ] T037 [P3] [] Test environment variable edge cases (file: tests/unit/config.rs)
  - Test whitespace in env vars (should trim)
  - Test empty env vars (should error)
  - Test missing env vars (should warn and continue)
  - Test special characters (should validate or error)

---

## Phase 10: Final Validation Against Spec

### T038 - Acceptance Criteria Validation
- [ ] T038 [P1] [US1] Validate US1 acceptance scenarios (file: specs/001-mcp-server-foundation/VALIDATION.md)
  - Run quickstart Scenario 1
  - Verify all 3 acceptance criteria pass
  - Document results

- [ ] T039 [P1] [US2] Validate US2 acceptance scenarios (file: specs/001-mcp-server-foundation/VALIDATION.md)
  - Run quickstart Scenario 2
  - Verify all 3 acceptance criteria pass
  - Document results

- [ ] T040 [P1] [US3] Validate US3 acceptance scenarios (file: specs/001-mcp-server-foundation/VALIDATION.md)
  - Run quickstart Scenario 3
  - Verify all 4 acceptance criteria pass
  - Document results

- [ ] T041 [P1] [] Validate all Success Criteria SC-001 through SC-008 (file: specs/001-mcp-server-foundation/VALIDATION.md)
  - Run performance benchmarks
  - Run security tests
  - Run error recovery tests
  - Integrate with Claude Desktop
  - Document all measurements
  - Create final validation report

- [ ] T042 [P1] [] Verify constitution compliance (file: specs/001-mcp-server-foundation/VALIDATION.md)
  - Security-First: Audit credential handling
  - MCP Protocol Compliance: Test with MCP Inspector
  - Type Safety: Verify no panics in error paths
  - Async-First: Verify no blocking operations
  - Document compliance for each principle

---

## Dependencies & Ordering

**Critical Path (Must Complete First):**
1. Phase 0 (T001-T004) → Project setup MUST complete before any code
2. Phase 1 (T005-T007) → Error types MUST exist before client/server
3. Phase 2 (T008-T010) → Config MUST exist before server init

**Parallel Work (Can Be Done Concurrently After Phase 1):**
- Phase 3 (Binance Client) + Phase 4 (MCP Server) are independent
- Phase 2 (Config) can overlap with Phase 3/4

**Sequential After Phase 5:**
- Phase 6 (Testing) requires all implementation phases
- Phase 7 (Docs) can start during Phase 6
- Phase 8-10 (Validation) must be last

**Blocking Tasks:**
- T002 blocks ALL code tasks (dependencies must be defined)
- T005-T007 block T011-T022 (errors needed for Result types)
- T015-T018 block T019-T022 (server needed for tools)
- T019-T022 block T023-T026 (tools needed for integration tests)

---

## Progress Tracking

**Total Tasks:** 42
**P1 Tasks:** 35 (83%)
**P2 Tasks:** 6 (14%)
**P3 Tasks:** 1 (3%)

**Phases:**
- Phase 0: 4 tasks (setup)
- Phase 1: 3 tasks (foundation)
- Phase 2: 3 tasks (config)
- Phase 3: 4 tasks (client)
- Phase 4: 4 tasks (server)
- Phase 5: 4 tasks (tools)
- Phase 6: 4 tasks (testing)
- Phase 7: 4 tasks (docs)
- Phase 8: 3 tasks (performance)
- Phase 9: 4 tasks (edge cases)
- Phase 10: 5 tasks (validation)

**Estimated Effort:** 2-3 days for single developer

**Implementation Command:** `/speckit.implement` (after review)

---

## Notes

- All file paths are relative to repository root
- Task IDs are unique per feature (T001-T042)
- Priority inherits from user story unless cross-cutting (no US)
- Tasks follow quickstart.md validation scenarios
- Constitution principles verified in Phase 10
- Manual testing with Claude Desktop required for SC-006

**Ready for:** `/speckit.implement` execution
