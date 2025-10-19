# Research: Streamable HTTP Transport Cleanup

**Feature**: 010-specify-scripts-bash | **Date**: 2025-10-18

## Overview

This research phase documents the technical decisions for removing legacy SSE transport code. Since the Streamable HTTP implementation is already working and verified, this is primarily a code cleanup task rather than new feature research.

## Technical Decisions

### Decision 1: Which Code to Remove

**Context**: The codebase currently contains two transport patterns:
1. **Legacy SSE**: GET `/sse` handshake → returns `X-Connection-ID` → POST `/messages` with header
2. **Streamable HTTP**: POST `/mcp` with `initialize` method → returns `Mcp-Session-Id` header → subsequent POSTs with header

**Decision**: Remove all legacy SSE GET handshake code while preserving Streamable HTTP implementation

**Rationale**:
- Streamable HTTP is the MCP 2025-03-26 specification standard
- ChatGPT integration is already verified working with POST `/mcp` only
- Dual-endpoint logic causes maintainability confusion
- 40% code reduction opportunity (removing ~400 lines from handlers)

**Alternatives Considered**:
1. **Keep both patterns**: Rejected - increases code complexity, testing burden, and maintenance cost
2. **Remove Streamable HTTP, keep SSE**: Rejected - SSE is deprecated spec, ChatGPT requires Streamable HTTP
3. **Gradual deprecation warnings**: Rejected - no known clients use old pattern, immediate removal preferred

### Decision 2: Backward Compatibility Strategy

**Context**: The codebase has multiple endpoints that might be in use:
- `/sse` (GET) - old handshake
- `/messages` (POST) - old message endpoint
- `/mcp` (GET + POST) - new Streamable HTTP endpoint
- `/mcp/sse` (GET) - custom path (legacy)
- `/mcp/message` (POST) - custom path (legacy)

**Decision**: Keep `/messages` POST endpoint for backward compatibility, remove all GET handshake endpoints

**Rationale**:
- `/messages` can be aliased to `/mcp` handler (zero cost, same session management)
- GET endpoints serve no purpose in Streamable HTTP (POST-only protocol)
- Minimal code to maintain one backward-compatible alias
- Provides migration path if any unknown clients exist

**Alternatives Considered**:
1. **Remove everything immediately**: Rejected - too aggressive, no migration period
2. **Keep all endpoints**: Rejected - defeats purpose of cleanup
3. **Add deprecation headers**: Rejected - extra complexity for unknown benefit (no known old clients)

### Decision 3: Session Management Changes

**Context**: Current session management supports both `X-Connection-ID` and `Mcp-Session-Id` headers

**Decision**: Remove `X-Connection-ID` validation code, keep only `Mcp-Session-Id`

**Rationale**:
- Session struct and HashMap storage unchanged (no migration needed)
- Single header validation path reduces code complexity
- No performance impact (same HashMap lookup)
- Follows Streamable HTTP specification exactly

**Alternatives Considered**:
1. **Support both headers**: Rejected - increases code paths, testing complexity
2. **Translate X-Connection-ID to Mcp-Session-Id**: Rejected - unnecessary adapter layer

### Decision 4: Router Configuration

**Context**: Router currently has 7 routes for SSE/MCP endpoints

**Decision**: Simplify to 3 routes:
- POST `/mcp` - primary Streamable HTTP endpoint
- POST `/messages` - backward compatibility alias
- GET `/health` - health check (keep unchanged)

**Rationale**:
- Removes 4 redundant routes (GET `/sse`, `/mcp/sse`, `/mcp/message`)
- Clearer routing table for developers
- Optional: GET `/mcp` could return server info (out of scope for this feature)

**Alternatives Considered**:
1. **Keep custom paths (/mcp/sse, /mcp/message)**: Rejected - non-standard, adds confusion
2. **Single endpoint (/mcp only)**: Rejected - breaking change for /messages users (if any)

### Decision 5: Error Response Format

**Context**: Current implementation already returns JSON-RPC 2.0 errors with correct codes

**Decision**: No changes to error handling - already compliant with spec

**Rationale**:
- Current error codes match Streamable HTTP spec (-32000, -32001, -32002)
- HTTP status codes (400, 404, 503) are appropriate
- This cleanup focuses on code removal, not error handling improvements

**Alternatives Considered**:
1. **Add more error codes**: Out of scope (spec defines only 3 codes)
2. **Change error messages**: Out of scope (user stories don't require this)

## Implementation Risks

### Risk 1: Breaking Unknown Clients

**Probability**: Low
**Impact**: Medium (client stops working)

**Mitigation**:
- Keep `/messages` endpoint for 1-2 releases before removal
- Document migration path in CHANGELOG
- Monitor Shuttle logs for 404s on removed endpoints

### Risk 2: Incomplete Code Removal

**Probability**: Low
**Impact**: Low (confusion remains)

**Mitigation**:
- Use grep/ripgrep to find all references: `sse_handshake`, `X-Connection-ID`
- Code review checklist verifying zero references
- Success criteria SC-002 explicitly measures this

### Risk 3: Test Coverage Gaps

**Probability**: Very Low
**Impact**: Medium (bugs in production)

**Mitigation**:
- Integration tests already verify POST `/mcp` workflow
- No new functionality to test (only removal)
- ChatGPT integration serves as live integration test

## Best Practices Applied

### Rust Code Cleanup Patterns

1. **Dead code elimination**: Use `#[allow(dead_code)]` warnings to detect unused functions
2. **Grep verification**: `rg "sse_handshake|X-Connection-ID"` to verify complete removal
3. **Compiler-driven refactoring**: Remove code, let compiler errors guide what else to update

### MCP Specification Compliance

1. **Streamable HTTP March 2025 spec**: Single POST endpoint, `Mcp-Session-Id` header
2. **JSON-RPC 2.0 errors**: Error codes -32000 (server error), -32001 (session not found), -32002 (missing header)
3. **Session lifecycle**: `initialize` → create session → validate session → expire on timeout

### Axum Router Best Practices

1. **Route consolidation**: Fewer routes = clearer routing table
2. **Backward compatibility**: Alias old endpoint to new handler (zero cost)
3. **Method routing**: `.route("/path", post(handler))` instead of method_router()

## Glossary

- **SSE Transport**: Server-Sent Events transport (MCP old spec, deprecated)
- **Streamable HTTP**: MCP transport protocol (March 2025 specification)
- **GET handshake**: Old pattern requiring GET request before POST messages
- **Mcp-Session-Id**: Session header used in Streamable HTTP (replaces X-Connection-ID)
- **JSON-RPC 2.0**: Request/response protocol used by MCP
- **Initialize method**: First MCP request that creates session

## References

- [MCP Streamable HTTP Specification](https://spec.modelcontextprotocol.io/specification/basic/transports/#streamable-http-with-server-sent-events-transport) (March 2025)
- [Axum Routing Documentation](https://docs.rs/axum/latest/axum/routing/index.html)
- [handlers_simple.rs](src:handlers_simple.rs:39-83) - Current `sse_handshake` implementation
- [handlers_simple.rs](src:handlers_simple.rs:92-168) - Current session validation logic
- [main.rs](src:main.rs:194-210) - Current router configuration

## Summary

This is a low-risk cleanup refactoring with clear benefits:
- ✅ 40% code reduction in handlers
- ✅ Clearer codebase (single transport pattern)
- ✅ Streamable HTTP spec compliance
- ✅ ChatGPT integration verified working
- ✅ No new dependencies or complexity

No unresolved technical questions remain. Ready to proceed to Phase 1 (design artifacts).
