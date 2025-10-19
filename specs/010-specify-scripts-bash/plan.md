# Implementation Plan: Streamable HTTP Transport Cleanup

**Branch**: `010-specify-scripts-bash` | **Date**: 2025-10-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/010-specify-scripts-bash/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Remove legacy SSE transport code (GET handshake endpoint, `X-Connection-ID` header) and consolidate to pure Streamable HTTP implementation (single POST `/mcp` endpoint, `Mcp-Session-Id` header). This is a cleanup refactoring with no new functionality - the Streamable HTTP implementation already works and is verified with ChatGPT integration.

## Technical Context

**Language/Version**: Rust 1.90+ (Edition 2024)
**Primary Dependencies**: axum 0.8+ (HTTP server), tokio 1.48.0 (async runtime), rmcp 0.8.1 (MCP SDK)
**Storage**: In-memory HashMap for session management (no changes)
**Testing**: cargo test (integration tests verify POST `/mcp` endpoint only)
**Target Platform**: Linux/macOS server (Shuttle.dev cloud deployment)
**Project Type**: Single project (MCP server binary)
**Performance Goals**: No change (existing session limit: 50 concurrent, 5s timeout)
**Constraints**: Must maintain backward compatibility for `/messages` endpoint temporarily
**Scale/Scope**: ~400 lines removed from handlers_simple.rs, router configuration simplified

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First ✅ PASS
- **No security changes**: This is code removal only, no authentication/API changes
- **Rate limiting unchanged**: Session management retains 50 concurrent limit
- **Error messages**: JSON-RPC errors already sanitized (no stack traces)

### II. Auto-Generation Priority ✅ PASS (N/A)
- **No code generation**: Manual refactoring of existing handler code
- **Rationale**: Removing deprecated patterns, no external API schema involved

### III. Modular Architecture ✅ PASS
- **Feature gates unchanged**: SSE feature remains, only internal cleanup
- **Module boundaries**: Changes isolated to `src/transport/sse/handlers_simple.rs` and `src/main.rs` router

### IV. Type Safety & Contract Enforcement ✅ PASS
- **No type changes**: Session types, JSON-RPC types remain identical
- **MCP contract preserved**: Still returns proper JSON-RPC 2.0 responses

### V. MCP Protocol Compliance ✅ PASS (CRITICAL)
- **Streamable HTTP spec**: Already implemented, this removes old SSE GET handshake
- **`initialize` lifecycle**: Preserved (creates session, returns `Mcp-Session-Id`)
- **`tools/list` and `tools/call`**: Unchanged, still work via POST `/mcp`
- **Backward compatibility**: `/messages` endpoint kept temporarily

### VI. Async-First Design ✅ PASS
- **No async changes**: Handlers remain async, session management unchanged

### VII. Machine-Optimized Development ✅ PASS
- **SpecKit workflow**: Following `/speckit.specify` → `/speckit.plan` → `/speckit.tasks`
- **Measurable success**: SC-002 (zero `sse_handshake` references), SC-004 (40% code reduction)

**GATE STATUS**: ✅ ALL GATES PASSED - No constitution violations, proceed to Phase 0

---

**POST-DESIGN RE-EVALUATION** (after Phase 1):

All constitution checks remain PASSED after design phase. No new violations introduced:
- ✅ Security: No changes to authentication, session limits, or error handling
- ✅ Type Safety: No changes to Session or JSON-RPC types
- ✅ MCP Compliance: Streamable HTTP spec fully preserved
- ✅ Async Design: No changes to async handlers

Design artifacts confirm this is a pure code cleanup with zero architectural changes.

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
├── transport/
│   └── sse/
│       ├── handlers_simple.rs   # ⚠️  MODIFIED: Remove sse_handshake function, X-Connection-ID validation
│       ├── session.rs            # ✅ NO CHANGE: Session manager unchanged
│       └── mod.rs                # ✅ NO CHANGE: Module exports
├── main.rs                       # ⚠️  MODIFIED: Remove GET /sse route from router
├── server/                       # ✅ NO CHANGE: MCP server core unchanged
└── tools/                        # ✅ NO CHANGE: Tool implementations unchanged

tests/
└── integration/                  # ⚠️  MODIFIED: Update tests to use POST /mcp only
```

**Structure Decision**: Single project structure. This is a focused refactoring with changes isolated to:
1. `src/transport/sse/handlers_simple.rs` - Remove `sse_handshake()` function and `X-Connection-ID` header validation
2. `src/main.rs` - Remove GET `/sse` route, keep GET `/mcp` optional (server info or 404)
3. Integration tests - Verify POST `/mcp` endpoint works without GET handshake

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations** - All constitution checks passed. This is a cleanup refactoring that reduces complexity by removing deprecated code patterns.

