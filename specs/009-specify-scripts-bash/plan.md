# Implementation Plan: SSE Transport for Cloud Deployment

**Branch**: `009-specify-scripts-bash` | **Date**: 2025-10-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/009-specify-scripts-bash/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add Server-Sent Events (SSE) transport protocol to enable remote HTTPS access for the MCP server. This allows deployment to Shuttle.dev cloud platform with automatic SSL/HTTPS provisioning, while maintaining backward compatibility with existing stdio transport. The SSE implementation will reuse existing Axum HTTP server infrastructure and all current MCP tool implementations.

## Technical Context

**Language/Version**: Rust 1.90+ (Edition 2024)
**Primary Dependencies**:
- rmcp 0.8.1 (MCP SDK with SSE support - NEEDS CLARIFICATION: verify SSE API stability)
- tokio 1.48.0 (async runtime)
- axum 0.8+ (HTTP server framework - already in project)
- tower 0.5+ / tower-http 0.6+ (middleware for SSE - NEEDS CLARIFICATION: SSE-specific middleware requirements)

**Storage**: N/A (SSE is stateless protocol, session state in-memory only)
**Testing**: cargo test + Binance Testnet API + in-memory mock SSE clients
**Target Platform**: Linux server (Shuttle.dev containerized deployment)
**Project Type**: Single project (Rust library + binary)
**Performance Goals**:
- SSE connection handshake < 500ms
- 50 concurrent SSE connections
- Tool call latency same as stdio mode (< 2s for market data queries)

**Constraints**:
- Zero manual HTTPS/SSL configuration (Shuttle handles TLS termination)
- Backward compatibility with stdio transport (no breaking changes)
- Reuse all existing tool implementations (no duplication)

**Scale/Scope**:
- 2 new SSE endpoints (`/mcp/sse`, `/mcp/message`)
- 1 new feature flag (`sse`)
- ~500 LOC estimated (SSE server wrapper + Shuttle integration)
- OAuth2 authentication: NEEDS CLARIFICATION (required for production or optional for initial MVP?)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First ✅ PASS
- **API Key Storage**: Shuttle secrets store (environment variables) - compliant
- **HMAC Signing**: Reuses existing BinanceClient auth - no changes needed
- **Rate Limiting**: Inherits existing GCRA rate limiter - SSE doesn't affect limits
- **Input Validation**: SSE transport layer doesn't change tool validation logic
- **Error Messages**: Existing error handling applies to SSE responses
- **Dependency Audit**: rmcp 0.8.1 passes `cargo audit` (verified in previous features)

**Status**: ✅ No security regressions. SSE is transport-only change.

### II. Auto-Generation Priority ⚠️ PARTIAL
- **SSE Server Code**: MANUAL implementation required (rmcp SDK provides primitives, not full server)
- **MCP Tool Handlers**: ✅ REUSED from existing code (no regeneration needed)
- **Binance API Models**: ✅ No changes (SSE doesn't affect data models)

**Justification**: rmcp SDK provides `SseServer` type but not full integration with Axum. Glue code is manual by design (per constitution § II).

**Status**: ✅ PASS - manual code limited to transport layer (allowed by constitution)

### III. Modular Architecture ✅ PASS
- **Feature Flag**: New `sse` feature in Cargo.toml
- **Independent Compilation**: SSE feature can be disabled without affecting stdio
- **Cross-Module Dependencies**: `sse` feature requires `http-api` feature (explicit in Cargo.toml)
- **Dynamic Tool Registration**: Existing MCP server already supports this (no changes needed)

**Status**: ✅ Fully compliant. SSE is optional feature like `orderbook` or `http-api`.

### IV. Type Safety & Contract Enforcement ✅ PASS
- **SSE Message Types**: Uses existing `rmcp::protocol::JsonRpcMessage` types
- **JSON Schema**: No new schemas (SSE wraps existing tool schemas)
- **Deserialization**: rmcp SDK handles SSE message parsing with proper error handling
- **Timestamps**: No SSE-specific timestamp handling needed

**Status**: ✅ No new type safety concerns. SSE reuses existing MCP protocol types.

### V. MCP Protocol Compliance ✅ PASS
- **Lifecycle**: SSE transport must implement same `initialize` → `initialized` flow as stdio
- **Tool Discovery**: `tools/list` endpoint works identically in SSE mode
- **Tool Execution**: `tools/call` via JSON-RPC 2.0 over SSE (protocol-compliant)
- **Progress Notifications**: Existing notification system works with SSE transport
- **Dual Transport**: Constitution requires "stdio (default) and streamable HTTP" - SSE satisfies HTTP requirement

**Status**: ✅ SSE is MCP-compliant transport. rmcp SDK ensures protocol correctness.

### VI. Async-First Design ✅ PASS
- **Tokio Runtime**: SSE server runs on existing Tokio runtime
- **Async API Calls**: No changes (transport layer doesn't affect Binance client)
- **WebSocket Streams**: No conflicts (SSE is HTTP, WebSocket is separate feature)
- **Rate Limiting**: Existing semaphore-based limiter works with SSE connections
- **Error Handling**: Existing `thiserror` + `async-trait` patterns apply

**Status**: ✅ SSE is async-native (built on Axum which uses Tokio).

### VII. Machine-Optimized Development ✅ PASS
- **Specification**: Created via `/speckit.specify` ✅
- **Machine-Readable Format**: spec.md has Given/When/Then, FR-###, SC-### ✅
- **Task Mapping**: Will be generated by `/speckit.tasks` after planning ✅
- **Tests-First**: Constitution allows "tests when required" - SSE integration tests recommended
- **Clarifications**: 3 NEEDS CLARIFICATION markers identified in Technical Context (will resolve in Phase 0)

**Status**: ✅ Workflow compliance verified.

---

**GATE DECISION**: ✅ **PASS** - All constitution principles satisfied. Proceed to Phase 0 research.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── transport/          # NEW: Transport layer implementations
│   ├── mod.rs
│   ├── stdio.rs       # Existing stdio transport
│   └── sse.rs         # NEW: SSE transport (feature-gated)
├── http/              # Existing HTTP API server
│   ├── mod.rs
│   └── server.rs      # Will add SSE routes here
├── server/            # Existing MCP server core
│   ├── mod.rs
│   ├── handler.rs     # Tool handlers (no changes)
│   └── types.rs
├── binance/           # Existing Binance API client (no changes)
├── orderbook/         # Existing orderbook features (no changes)
└── main.rs            # Binary entry point (add SSE mode flag)

Shuttle.toml           # NEW: Shuttle deployment config
.shuttle/              # NEW: Shuttle runtime files (gitignored)
  └── secrets.toml     # Gitignored secrets file

tests/
├── integration/
│   └── sse_transport_test.rs  # NEW: SSE integration tests
└── unit/
    └── transport/
        └── sse_test.rs        # NEW: SSE unit tests
```

**Structure Decision**: Single Rust project with feature-gated SSE module. The SSE transport will be implemented in `src/transport/sse.rs` behind the `sse` feature flag. Integration with existing Axum HTTP server happens in `src/http/server.rs` where SSE endpoints will be conditionally compiled when both `http-api` and `sse` features are enabled.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

