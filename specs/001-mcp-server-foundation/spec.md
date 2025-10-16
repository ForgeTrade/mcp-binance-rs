# Feature Specification: MCP Server Foundation

**Feature Branch**: `001-mcp-server-foundation`
**Created**: 2025-10-16
**Status**: Draft
**Input**: User description: "Базовый MCP сервер для Binance. Минимальный работающий каркас со следующими возможностями: 1) MCP server реализация используя rmcp crate 2) Stdio transport для локального использования 3) Конфигурация API ключей через переменные окружения 4) Базовая error handling инфраструктура 5) Один демонстрационный tool get_server_time который возвращает текущее время сервера Binance. Цель: создать надежный фундамент для всех последующих функций следуя всем принципам конституции особенно Security-First и MCP Protocol Compliance."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - AI Assistant Initialization (Priority: P1)

An AI assistant (Claude, ChatGPT, or similar) needs to connect to the Binance MCP server and discover available tools for interacting with the Binance exchange.

**Why this priority**: Without a working MCP server that AI clients can connect to, no Binance functionality is accessible. This is the foundational requirement.

**Independent Test**: Can be fully tested by starting the MCP server via stdio transport, sending the MCP initialize handshake, and verifying the server responds with its capabilities. No Binance API interaction required.

**Acceptance Scenarios**:

1. **Given** an AI assistant wants to use Binance services, **When** it starts the MCP server via stdio and sends an initialize request, **Then** the server responds with protocol version, capabilities, and server information
2. **Given** the MCP server is initialized, **When** the AI assistant requests available tools via tools/list, **Then** the server returns at least one tool (get_server_time) with complete JSON Schema
3. **Given** the MCP server has no API keys configured, **When** it starts, **Then** it starts successfully and logs a warning that authenticated tools will not be available

---

### User Story 2 - Binance Time Synchronization (Priority: P1)

An AI assistant needs to verify connectivity with Binance and ensure time synchronization for future authenticated requests.

**Why this priority**: Time synchronization is critical for all authenticated Binance API calls. The get_server_time tool validates both connectivity and provides the foundation for timestamp-based request signing.

**Independent Test**: Can be tested by calling the get_server_time tool through the MCP server and verifying it returns Binance server time in milliseconds.

**Acceptance Scenarios**:

1. **Given** the MCP server is initialized, **When** the AI assistant calls the get_server_time tool with no parameters, **Then** the server returns Binance server time as a Unix timestamp in milliseconds
2. **Given** the Binance API is unreachable, **When** the get_server_time tool is called, **Then** the server returns a structured error with type "connection_error" and a user-friendly message
3. **Given** the get_server_time tool returns successfully, **When** compared to local system time, **Then** the difference is logged for time synchronization awareness

---

### User Story 3 - Secure API Key Management (Priority: P1)

A user configures their Binance API keys securely via environment variables, and the MCP server loads them without exposing sensitive data in logs or errors.

**Why this priority**: Security-First principle requires safe credential handling from the start. All future authenticated features depend on this foundation.

**Independent Test**: Can be tested by setting environment variables with test credentials, starting the server, and verifying: (1) credentials are loaded, (2) credentials never appear in INFO-level logs, (3) error messages don't expose key material.

**Acceptance Scenarios**:

1. **Given** valid API credentials are set via BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables, **When** the server starts, **Then** it loads the credentials and logs "API credentials configured" without showing the actual values
2. **Given** no API credentials are configured, **When** the server starts, **Then** it starts successfully and logs "No API credentials configured; authenticated features disabled"
3. **Given** API credentials are configured, **When** any error occurs during initialization, **Then** error messages do not contain credential material or signatures
4. **Given** the server is running with credentials, **When** debug logging is enabled, **Then** credentials may appear in DEBUG logs with explicit "SENSITIVE DATA" warnings

---

### Edge Cases

- What happens when Binance API rate limits are hit during get_server_time?
  - Server returns error with type "rate_limit" and includes retry-after information if available
- What happens when invalid JSON is sent to the MCP server?
  - Server returns JSON-RPC 2.0 parse error with appropriate error code
- What happens when the MCP client disconnects unexpectedly?
  - Server detects stdio closure and shuts down gracefully, logging disconnect event
- What happens when environment variables contain whitespace or special characters?
  - Server trims whitespace and validates format; rejects invalid characters with clear error message
- What happens when get_server_time is called while the server is still initializing?
  - Server returns error with type "not_ready" indicating initialization incomplete
- What happens when multiple environment variable formats are used (e.g., BINANCE_API_KEY vs. API_KEY)?
  - Server documents the canonical format and only uses BINANCE_API_KEY and BINANCE_SECRET_KEY

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST implement MCP protocol lifecycle: initialize → capability negotiation → initialized notification
- **FR-002**: System MUST support stdio transport for local process communication
- **FR-003**: System MUST load Binance API credentials from BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables
- **FR-004**: System MUST start successfully when no credentials are configured, with authenticated features disabled
- **FR-005**: System MUST expose a get_server_time tool that returns Binance server time via /api/v3/time endpoint
- **FR-006**: System MUST respond to tools/list requests with accurate JSON Schema for all available tools
- **FR-007**: System MUST respond to tools/call requests with structured responses containing result data or error information
- **FR-008**: System MUST validate all MCP protocol messages and reject invalid requests with appropriate JSON-RPC 2.0 errors
- **FR-009**: System MUST use structured error types (connection_error, rate_limit, parse_error, invalid_request, etc.) for all failures
- **FR-010**: System MUST log at appropriate levels: INFO for lifecycle events, WARN for recoverable errors, ERROR for critical failures
- **FR-011**: System MUST never log sensitive data (API keys, signatures, secrets) at INFO or WARN levels
- **FR-012**: System MUST use Tokio async runtime for all I/O operations
- **FR-013**: System MUST enforce timeouts on all external API calls (default: 10 seconds)
- **FR-014**: System MUST handle Binance API errors gracefully, mapping HTTP status codes to error types
- **FR-015**: System MUST implement exponential backoff for rate limit errors (429 status)

### Key Entities

- **MCP Server**: The main server instance that implements MCP protocol handlers and manages tool routing
- **Tool Router**: Routes MCP tool calls to appropriate Binance API handlers based on tool name
- **Binance Client**: HTTP client for making requests to Binance REST API endpoints
- **Credentials**: Secure container for API key and secret, loaded from environment variables
- **Error Response**: Structured error format containing error type, message, and optional metadata

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: AI assistants can successfully initialize and discover tools in under 500 milliseconds
- **SC-002**: The get_server_time tool returns accurate Binance server time within 100 milliseconds of request
- **SC-003**: Server handles 100 sequential tool calls without memory leaks or performance degradation
- **SC-004**: Zero sensitive data exposure in logs during 1000 operations with credentials configured
- **SC-005**: Server recovers gracefully from Binance API failures with user-friendly error messages in 100% of error cases
- **SC-006**: MCP protocol compliance validated by successful integration with Claude Desktop or equivalent MCP client
- **SC-007**: Server startup time under 1 second with credentials configured
- **SC-008**: Error messages guide users to resolution in 90% of configuration mistakes (missing env vars, invalid format, etc.)

## Assumptions

- Users have basic command-line knowledge to set environment variables
- Binance API endpoints are available at api.binance.com (not testnet)
- Users will interact with the server through MCP-compatible AI assistants (Claude Desktop, VS Code extensions, etc.)
- Stdio transport is sufficient for initial release; HTTP transport can be added later
- The rmcp crate provides stable MCP protocol implementation (version 0.1.x or later)
- Rust toolchain (1.75+) is available in the user's environment
- Users understand that get_server_time requires internet connectivity
- Default Binance rate limits apply (1200 weight per minute); no special rate limit agreements

## Out of Scope

- WebSocket streaming connections (future feature)
- Trading operations (order placement, cancellation, etc.)
- Multiple API key profiles or key rotation
- HTTP/SSE transport for remote access
- Rate limit tracking and weight management
- Account information queries beyond server time
- Historical data queries
- Market data subscriptions
- Authentication methods beyond HMAC SHA256
- Configuration files (only environment variables)
- Interactive credential prompts
- Credential encryption at rest
