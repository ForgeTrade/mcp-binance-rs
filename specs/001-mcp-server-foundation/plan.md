# Implementation Plan: MCP Server Foundation

**Branch**: `001-mcp-server-foundation` | **Date**: 2025-10-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-mcp-server-foundation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a minimal but complete MCP server for Binance that enables AI assistants to interact with Binance exchange via the Model Context Protocol. The server provides secure API credential management via environment variables, implements the full MCP lifecycle (initialize → tool discovery → tool execution), and exposes a single demonstration tool (get_server_time) that validates Binance API connectivity. Foundation for all future Binance functionality with strict adherence to Security-First and MCP Protocol Compliance principles.

## Technical Context

**Language/Version**: Rust 1.75+ (Edition 2021)
**Primary Dependencies**:
- rmcp (0.1.x+) - MCP protocol implementation
- tokio (1.x) - Async runtime (multi-threaded default flavor for concurrent request handling)
- reqwest (0.11.x) - HTTP client for Binance API
- serde / serde_json - JSON serialization
- thiserror - Error handling
- schemars - JSON Schema generation for MCP tools
- tracing / tracing-subscriber - Structured logging

**Storage**: N/A (stateless server; credentials from environment only)
**Testing**: cargo test (unit tests), integration tests via MCP client simulation
**Target Platform**: Cross-platform (Linux, macOS, Windows) - stdio-based local process
**Project Type**: Single (command-line MCP server binary)
**Performance Goals**:
- <500ms MCP initialization handshake
- <100ms Binance API calls (get_server_time)
- <1s server startup with credentials loaded

**Constraints**:
- Stdio transport only (no HTTP/SSE in foundation)
- No persistent state or configuration files
- Must handle graceful shutdown on stdio closure
- Must respect Binance rate limits (1200 weight/min default)
- Memory footprint <50MB idle

**Scale/Scope**:
- Single user (local process)
- 1 MCP tool initially (get_server_time)
- Foundation for ~50+ future tools across spot/margin/futures
- No concurrency limit (Tokio handles parallelism)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First (NON-NEGOTIABLE)
- ✅ **PASS**: API keys via environment variables (BINANCE_API_KEY, BINANCE_SECRET_KEY) - FR-003
- ✅ **PASS**: No sensitive data in INFO/WARN logs - FR-011
- ✅ **PASS**: HMAC SHA256 signing for authenticated requests (foundation; authenticated tools future)
- ✅ **PASS**: Rate limiting with exponential backoff - FR-015
- ✅ **PASS**: Input validation before API calls (tool parameter validation)
- ✅ **PASS**: Error messages sanitized - FR-009
- ✅ **PASS**: Dependency security audits via cargo audit (pre-commit)

### II. Auto-Generation Priority
- ✅ **PASS**: rmcp crate provides auto-generated MCP protocol handlers
- ⚠️ **PARTIAL**: Binance API client hand-written for get_server_time (single endpoint)
  - **Justification**: Foundation phase focuses on infrastructure. Future features will use binance-connector-rust (auto-generated from OpenAPI spec) per Constitution Principle II.
  - **Deferred**: Full auto-generation integration planned for feature 002+ (spot market data tools)

### III. Modular Architecture
- ✅ **PASS**: Cargo workspace structure prepared for future feature modules
- ✅ **PASS**: Feature gates planned: `spot` (default), `server` (default), `transport-stdio` (default)
- ⚠️ **PARTIAL**: Single binary for foundation; modular expansion in future features
  - **Justification**: Foundation establishes pattern; future features add modules (margin, futures, etc.)

### IV. Type Safety & Contract Enforcement
- ✅ **PASS**: Rust type system enforces correctness
- ✅ **PASS**: JSON Schema for MCP tool inputs/outputs via schemars - FR-006
- ✅ **PASS**: Deserialization failures treated as errors - FR-008
- ⚠️ **DEFER**: Timestamp newtypes and filter validation deferred to authenticated tools (future features)

### V. MCP Protocol Compliance (NON-NEGOTIABLE)
- ✅ **PASS**: Full MCP lifecycle: initialize → capability negotiation → initialized - FR-001
- ✅ **PASS**: tools/list with JSON Schema - FR-006
- ✅ **PASS**: tools/call with structured responses - FR-007
- ✅ **PASS**: Stdio transport - FR-002
- ✅ **PASS**: JSON-RPC 2.0 error handling - FR-008
- ⚠️ **DEFER**: HTTP/SSE transport out of scope (foundation); tools/list_changed notifications (no dynamic registration yet)
- ⚠️ **DEFER**: Progress notifications (get_server_time <2s; not needed)

### VI. Async-First Design
- ✅ **PASS**: Tokio async runtime - FR-012
- ✅ **PASS**: Async Binance API calls with timeout - FR-013
- ✅ **PASS**: thiserror for error propagation - FR-009
- ⚠️ **DEFER**: WebSocket streams and rate-limiter semaphore deferred to streaming features

### VII. Machine-Optimized Development
- ✅ **PASS**: Feature added via /speckit.specify workflow
- ✅ **PASS**: Spec with Given/When/Then scenarios, FR-###, SC-###
- ✅ **PASS**: Tasks will map to task IDs in tasks.md (Phase 2)
- ⚠️ **DEFER**: TDD cycle optional per spec (no tests explicitly required)

**Overall Assessment**: ✅ **PASSES** constitution gates with justified partial implementations

**Violations Requiring Justification**: None (all partials are deferred future enhancements documented below)

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
mcp-binance-rs/
├── Cargo.toml                 # Workspace root
├── Cargo.lock
├── .gitignore
├── README.md
│
├── src/                       # Main server binary
│   ├── main.rs                # Entry point: MCP server startup
│   ├── lib.rs                 # Library exports
│   ├── server/                # MCP server implementation
│   │   ├── mod.rs
│   │   ├── handler.rs         # ServerHandler trait impl
│   │   └── tool_router.rs     # Tool routing logic
│   ├── tools/                 # MCP tool implementations
│   │   ├── mod.rs
│   │   └── get_server_time.rs # get_server_time tool
│   ├── binance/               # Binance API client
│   │   ├── mod.rs
│   │   ├── client.rs          # HTTP client wrapper
│   │   └── types.rs           # API response types
│   ├── config/                # Configuration management
│   │   ├── mod.rs
│   │   └── credentials.rs     # Env var loading
│   └── error.rs               # Error types & conversions
│
├── tests/                     # Integration tests
│   └── integration/
│       ├── mcp_lifecycle.rs   # MCP initialization tests
│       └── server_time.rs     # get_server_time tool tests
│
├── docs/                      # Documentation
│   └── codegen.md             # Code generation guide (placeholder)
│
└── specs/                     # SpecKit feature documentation
    └── 001-mcp-server-foundation/
        ├── spec.md
        ├── plan.md            # This file
        ├── research.md        # Phase 0 output
        ├── data-model.md      # Phase 1 output
        ├── quickstart.md      # Phase 1 output
        └── contracts/         # Phase 1 output (MCP tool schemas)
```

**Structure Decision**: Single project layout selected. This is a command-line MCP server binary with no frontend/mobile components. The structure follows Rust conventions:
- `src/main.rs` for binary entry point
- `src/lib.rs` for library exports (allows testing internal modules)
- Modular organization by concern: `server/` (MCP), `tools/` (tool impls), `binance/` (API client), `config/` (env vars)
- Integration tests in `tests/` directory (separate from unit tests)
- Future features will add modules under `src/` (e.g., `src/spot/`, `src/margin/`) with Cargo feature gates

## Complexity Tracking

*No violations requiring justification. All constitution checks pass or are explicitly deferred to future features per Foundation scope.*

