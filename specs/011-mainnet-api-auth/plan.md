# Implementation Plan: Mainnet Support with Secure API Key Authentication

**Branch**: `011-mainnet-api-auth` | **Date**: 2025-10-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/011-mainnet-api-auth/spec.md`

## Summary

Enable per-session API credential configuration for Binance mainnet and testnet environments. Users configure credentials via MCP tools (`configure_credentials`, `get_credentials_status`, `revoke_credentials`) that are stored session-scoped in memory with no persistence to disk. Credentials are validated synchronously for format (64 char alphanumeric) and asynchronously by Binance API on first tool call. All errors return structured JSON with machine-readable error codes for programmatic handling.

**Primary Approach**: Extend SessionManager from Feature 010 to store Credentials struct per Mcp-Session-Id. Refactor BinanceClient to support per-session credentials instead of global environment variables. Add 3 new MCP tools for credential management.

## Technical Context

**Language/Version**: Rust 1.90+ (Edition 2024)
**Primary Dependencies**:
- rmcp 0.8.1 (MCP SDK with #[tool] macros)
- tokio 1.48.0 (async runtime for session management)
- serde 1.0.228 + serde_json 1.0.145 (structured error serialization)
- chrono 0.4 (timestamp for configured_at field)
- regex 1.11+ (format validation for API key/secret)

**Storage**: In-memory HashMap<String, Credentials> in SessionManager (no persistence)
**Testing**: cargo test with integration tests for session isolation and credential lifecycle
**Target Platform**: Linux server (Shuttle.dev deployment) + local stdio mode
**Project Type**: Single Rust project with feature-gated modules

**Performance Goals**:
- Credential configuration: <100ms (FR-SC-004)
- Format validation: <10ms synchronous (SC-007)
- Unconfigured error fast-fail: <50ms (SC-006)
- Zero latency overhead for public tools (NFR-004)

**Rate Limiting**: FR-012 rate limiting (1200 req/min signed, 6000 req/min public) is satisfied by existing BinanceClient rate limiter implemented in prior features. Credential-based requests use same infrastructure. No additional tasks required.

**Constraints**:
- No disk persistence (FR-004, security requirement)
- Session isolation enforced via Mcp-Session-Id (FR-001)
- Max 50 concurrent sessions from Feature 010 (memory safety)
- Credentials cleared immediately on session end (FR-003)

**Non-Functional Requirement Coverage**:
- NFR-001 (secure memory): Satisfied by Rust's Arc<RwLock> ownership + T010 session cleanup
- NFR-004 (zero latency public tools): Satisfied by design - public tools skip credential lookup entirely
- NFR-005 (deterministic errors): Satisfied by CredentialError enum + to_json() standardized formatting

**Scale/Scope**:
- 3 new MCP tools (configure_credentials, get_credentials_status, revoke_credentials)
- 1 new error type (CredentialError) with 6 error codes
- 2 modules modified (SessionManager, BinanceClient)
- 7 account/trading tools refactored to use session credentials
- ~500 lines of new code estimated

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First (NON-NEGOTIABLE)

✅ **PASS** - API keys stored in session-scoped memory only (FR-004)
✅ **PASS** - API secrets never logged (NFR-002)
✅ **PASS** - Input validation before storage (FR-010, format validation)
✅ **PASS** - Error messages use structured format, no sensitive data exposure (FR-009, FR-013)
✅ **PASS** - HMAC SHA256 signing unchanged (existing BinanceClient logic)

### II. Auto-Generation Priority

✅ **PASS** - No auto-generation applicable (session management is glue logic)
✅ **PASS** - MCP tool schemas auto-generated via schemars from Rust structs

### III. Modular Architecture

✅ **PASS** - No new feature gates added (credential management is core functionality)
✅ **PASS** - Credentials struct reusable across transport modes (stdio, SSE)

### IV. Type Safety & Contract Enforcement

✅ **PASS** - Environment enum enforces testnet/mainnet values (FR-002)
✅ **PASS** - Credentials struct with typed fields (api_key: String, configured_at: DateTime<Utc>)
✅ **PASS** - Error enum for CredentialError with exhaustive error codes (FR-013)
✅ **PASS** - JSON Schema via #[derive(schemars::JsonSchema)] for all MCP tool parameters

### V. MCP Protocol Compliance (NON-NEGOTIABLE)

✅ **PASS** - 3 new tools follow MCP tool lifecycle (tools/list, tools/call)
✅ **PASS** - Structured error responses comply with MCP error format (JSON-RPC 2.0)
✅ **PASS** - No dynamic tool registration (credentials are always available, gated by session state)

### VI. Async-First Design

✅ **PASS** - SessionManager already async (from Feature 010)
✅ **PASS** - Credential storage/retrieval via async methods
✅ **PASS** - Format validation is sync (regex check), API validation is async (first tool call)

### VII. Machine-Optimized Development

✅ **PASS** - Feature added via `/speckit.specify` workflow
✅ **PASS** - Specification uses Given/When/Then scenarios, numbered requirements (FR-001 through FR-013)
✅ **PASS** - Measurable success criteria (SC-001 through SC-007)
✅ **PASS** - All ambiguities resolved via `/speckit.clarify` (2 clarification questions)

**Gate Result**: ✅ **ALL GATES PASSED** - Proceed to Phase 0

## Project Structure

### Documentation (this feature)

```
specs/011-mainnet-api-auth/
├── spec.md              # Feature specification with clarifications
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (session isolation patterns)
├── data-model.md        # Phase 1 output (Credentials, Environment, CredentialError entities)
├── quickstart.md        # Phase 1 output (configure credentials → call get_account_info)
├── contracts/           # Phase 1 output (MCP tool JSON schemas)
│   ├── configure_credentials.json
│   ├── get_credentials_status.json
│   └── revoke_credentials.json
├── checklists/          # Quality validation checklists
│   └── requirements.md  # Spec completeness checklist (62 items, all passed)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created yet)
```

### Source Code (repository root)

```
src/
├── binance/
│   └── client.rs        # [MODIFIED] Add per-session credential support
├── error/
│   └── mod.rs           # [MODIFIED] Add CredentialError type with error codes
├── server/
│   ├── mod.rs           # [UNCHANGED] MCP server core
│   └── tool_router.rs   # [MODIFIED] Add 3 new credential management tools
├── tools/
│   └── credentials.rs   # [NEW] Credential management tool handlers
├── transport/
│   └── sse/
│       ├── session.rs   # [MODIFIED] SessionManager stores Credentials per session
│       └── handlers_simple.rs # [MODIFIED] Pass session credentials to BinanceClient
└── main.rs              # [UNCHANGED] Entry point

tests/
├── integration/
│   ├── credentials_test.rs  # [NEW] Session isolation, lifecycle tests
│   └── sse_transport_test.rs # [MODIFIED] Add credential configuration scenarios
└── unit/
    └── credentials_validation_test.rs # [NEW] Format validation unit tests
```

**Structure Decision**: Single Rust project with modular feature-gated design. Credential management is core functionality (no feature gate), integrated into existing session management (Feature 010 SSE transport) and MCP server (tool_router.rs). New module `src/tools/credentials.rs` contains tool logic, while `src/transport/sse/session.rs` stores credentials in SessionManager.

## Complexity Tracking

*No complexity violations - all gates passed. This table is empty.*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none) | N/A | N/A |

**Rationale for Empty Table**: Feature aligns perfectly with constitution:
- Security-first: No disk persistence, session isolation
- Modular: Extends existing session management without new feature gates
- Type-safe: Rust enums for Environment, structured error types
- MCP-compliant: Standard tool lifecycle with JSON schemas
- Async-first: Builds on async SessionManager from Feature 010
- Machine-optimized: Specification-driven development via speckit workflow

No principle violations detected. Implementation follows established patterns from Feature 010 (session management) with minimal new complexity.
