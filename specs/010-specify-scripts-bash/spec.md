# Feature Specification: Streamable HTTP Transport Cleanup

**Feature Branch**: `010-streamable-http-cleanup`
**Created**: 2025-10-18
**Status**: Draft
**Input**: Refactor SSE transport to pure Streamable HTTP (MCP 2025-03-26 spec)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - ChatGPT MCP Integration (Priority: P1)

ChatGPT users connect to the Binance MCP server using the Streamable HTTP transport protocol (March 2025 specification). The server handles all MCP requests through a single `/mcp` endpoint without requiring a separate handshake step.

**Why this priority**: This is the core functionality that enables ChatGPT integration, which was the primary goal of the recent work. Without this, the server cannot be used with ChatGPT.

**Independent Test**: Can be fully tested by connecting ChatGPT to the `/mcp` endpoint and executing tool calls. Delivers immediate value by enabling ChatGPT connectors.

**Acceptance Scenarios**:

1. **Given** no prior connection, **When** ChatGPT sends POST `/mcp` with `initialize` method, **Then** server creates session and returns `Mcp-Session-Id` header
2. **Given** valid `Mcp-Session-Id` header, **When** ChatGPT sends POST `/mcp` with `tools/list` method, **Then** server returns list of available tools
3. **Given** valid `Mcp-Session-Id` header, **When** ChatGPT sends POST `/mcp` with `tools/call` method, **Then** server executes tool and returns results in MCP content array format

---

### User Story 2 - Maintainable Codebase (Priority: P2)

Developers can understand and modify the transport implementation without confusion from legacy SSE code. The codebase contains only Streamable HTTP implementation with clear documentation.

**Why this priority**: Essential for long-term maintainability but doesn't affect current functionality. Can be done after verifying P1 works.

**Independent Test**: Code review shows no references to old SSE handshake patterns, GET endpoint removed, session management simplified.

**Acceptance Scenarios**:

1. **Given** transport module code, **When** developer reviews handlers, **Then** only `/mcp` POST endpoint exists for message handling
2. **Given** session management code, **When** developer reads implementation, **Then** only `Mcp-Session-Id` header validation logic exists
3. **Given** router configuration, **When** developer checks endpoints, **Then** old `/sse` GET endpoint is removed

---

### User Story 3 - Proper Error Responses (Priority: P3)

When clients send invalid requests, they receive JSON-RPC 2.0 error responses with appropriate HTTP status codes according to Streamable HTTP spec.

**Why this priority**: Improves developer experience but not critical for basic functionality.

**Independent Test**: Send various invalid requests and verify error responses match specification.

**Acceptance Scenarios**:

1. **Given** missing `Mcp-Session-Id` header, **When** client sends non-initialize request, **Then** server returns 400 with JSON-RPC error code -32002
2. **Given** invalid `Mcp-Session-Id` value, **When** client sends request, **Then** server returns 404 with JSON-RPC error code -32001
3. **Given** exceeded session limit, **When** client sends initialize, **Then** server returns 503 with JSON-RPC error code -32000

---

### Edge Cases

- What happens when session expires mid-conversation? (Client receives 404, must re-initialize)
- How does system handle rapid initialize requests? (Each gets unique session ID up to 50 concurrent limit)
- What if client sends tools/call without initialize? (400 error: Missing Mcp-Session-Id)
- How does GET to `/mcp` behave? (Not used in Streamable HTTP - could return server info or 404)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST handle all MCP JSON-RPC requests through POST `/mcp` endpoint only
- **FR-002**: System MUST create new session on first `initialize` request and return `Mcp-Session-Id` header
- **FR-003**: System MUST validate `Mcp-Session-Id` header for all non-initialize requests
- **FR-004**: System MUST remove old GET `/sse` handshake endpoint
- **FR-005**: System MUST remove `X-Connection-ID` header validation code (see FR-009 for related function removal)
- **FR-006**: System MUST use `Mcp-Session-Id` header for session tracking instead of `X-Connection-ID`
- **FR-007**: System MUST return JSON-RPC 2.0 error responses with appropriate codes for invalid requests
- **FR-008**: System MUST support both `application/json` and `text/event-stream` response formats based on `Accept` header
- **FR-009**: System MUST remove unused `sse_handshake` function from handlers (generates `X-Connection-ID` header, related to FR-005)
- **FR-010**: System MUST maintain backward compatibility for `/messages` POST endpoint (map to `/mcp` behavior)

### Key Entities

- **Session**: Represents active MCP client connection, identified by `Mcp-Session-Id`, tracks activity timestamp, limited to 50 concurrent sessions
- **JSON-RPC Request**: Standard JSON-RPC 2.0 message with method, params, id fields
- **JSON-RPC Response**: Standard JSON-RPC 2.0 response with result or error, includes `Mcp-Session-Id` header for initialize requests

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: ChatGPT successfully connects and executes tools without requiring GET handshake
- **SC-002**: Codebase contains zero references to old `sse_handshake` GET endpoint pattern (verified by `rg "sse_handshake|X-Connection-ID" src/` returning zero matches)
- **SC-003**: All integration tests pass using only POST `/mcp` endpoint
- **SC-004**: Session management code is 40% smaller (baseline: handlers_simple.rs ~526 lines → target: ~400 lines, verified by `wc -l`)
- **SC-005**: Developer can understand transport flow in under 5 minutes of code review (verified by external reviewer completing checklist: identify endpoints, trace session creation, understand error handling)

## Scope *(mandatory)*

### In Scope

- Remove `sse_handshake` GET endpoint and function
- Update router to remove GET `/sse` route
- Remove `X-Connection-ID` header validation
- Consolidate session management to use only `Mcp-Session-Id`
- Update comments and documentation to reflect Streamable HTTP only
- Keep backward compatibility route `/messages` → `/mcp`
- Update CHANGELOG.md documenting removed endpoints and migration path for any existing clients

### Out of Scope

- Changing session storage mechanism (still using in-memory HashMap)
- Adding DELETE `/mcp` for explicit session termination
- Implementing streaming SSE responses for long-running operations
- WebSocket transport support
- Changes to tool implementations (search, fetch, get_ticker, etc.)

## Assumptions *(mandatory)*

- Current implementation correctly handles `initialize` method
- Session timeout logic (activity tracking) remains unchanged
- 50 concurrent session limit is sufficient
- Backward compatibility for old `/messages` endpoint is required temporarily
- ChatGPT only uses POST requests, not GET
- All clients will adopt Streamable HTTP spec (no clients still using old SSE GET handshake)

## Dependencies

- No external dependencies
- Existing session management infrastructure
- Existing JSON-RPC routing logic

## Risks

- **Low risk**: Breaking existing Claude Desktop clients (if any exist)
  - *Mitigation*: Keep `/messages` endpoint for backward compatibility temporarily
- **Low risk**: Incomplete removal of old code causing confusion
  - *Mitigation*: Thorough code review and grep for `sse_handshake`, `X-Connection-ID`

## Glossary

**MCP Transport Terminology** (to avoid confusion between old and new patterns):

- **SSE Transport** (deprecated): Old MCP transport pattern requiring GET `/sse` handshake that returns `X-Connection-ID` header, followed by POST `/messages` requests with that header. This pattern is being removed in this feature.

- **Streamable HTTP Transport** (current spec): MCP transport protocol specified in March 2025 revision. Uses single POST `/mcp` endpoint for all requests. First request (`initialize` method) creates session and returns `Mcp-Session-Id` header. Subsequent requests include `Mcp-Session-Id` header for validation.

- **Session ID Headers**:
  - `X-Connection-ID`: Header used in deprecated SSE transport (being removed)
  - `Mcp-Session-Id`: Header used in Streamable HTTP transport (current standard)

- **Handshake**: In old SSE pattern, required GET request before any POST requests. Streamable HTTP has no separate handshake - session creation happens in first POST request (`initialize` method).

