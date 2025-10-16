# Data Model: MCP Server Foundation

**Feature**: MCP Server Foundation
**Date**: 2025-10-16
**Status**: Foundation Phase

## Overview

The MCP Server Foundation is a stateless, event-driven system with no persistent storage. All data structures are transient, existing only in memory during the server's lifecycle. This document defines the core entities and their relationships for MCP protocol handling and Binance API integration.

## Core Entities

### 1. ServerInfo

**Purpose**: Represents the MCP server's identity and capabilities during initialization.

**Attributes**:
- `name: String` - Server identifier ("mcp-binance-server")
- `version: String` - Semantic version (e.g., "0.1.0")
- `protocol_version: String` - MCP protocol version (e.g., "2024-11-05")
- `capabilities: ServerCapabilities` - Advertised server capabilities

**Lifecycle**: Created once at startup, immutable throughout server lifetime.

**Validation Rules**:
- `name` must be non-empty
- `version` must follow semver format
- `protocol_version` must match supported MCP versions

**Related Entities**: None (top-level entity)

---

### 2. ServerCapabilities

**Purpose**: Declares what the MCP server can do (tool support, resource support, etc.).

**Attributes**:
- `tools: Option<ToolsCapability>` - Tool support declaration
  - `list_changed: bool` - Whether server sends tool list change notifications
- `resources: Option<ResourcesCapability>` - Resource support (unused in foundation)
- `prompts: Option<PromptsCapability>` - Prompt support (unused in foundation)

**Lifecycle**: Created with ServerInfo at startup, immutable.

**Validation Rules**:
- `tools` must be `Some` with `list_changed: false` (no dynamic tool registration yet)
- `resources` and `prompts` are `None` (out of scope)

**Related Entities**: ServerInfo (parent)

---

### 3. Credentials

**Purpose**: Secure container for Binance API credentials loaded from environment variables.

**Attributes**:
- `api_key: SecretString` - Binance API key (public identifier)
- `secret_key: SecretString` - Binance secret key (private signing key)

**Lifecycle**:
- Created once at startup via environment variable loading
- Stored in server state, never serialized
- Dropped on server shutdown

**Validation Rules**:
- Both fields must be non-empty strings
- `api_key` typically 64 hex chars
- `secret_key` typically 64 hex chars
- Whitespace trimmed on load

**Security Constraints**:
- Never logged at INFO/WARN levels
- Debug logs must prefix with `[SENSITIVE DATA]`
- Never included in error messages
- Not serialized to JSON

**Related Entities**: BinanceClient (consumer)

---

### 4. Tool

**Purpose**: Represents an MCP tool definition for tool discovery.

**Attributes**:
- `name: String` - Tool identifier ("get_server_time")
- `description: String` - Human-readable tool purpose
- `input_schema: JsonSchema` - JSON Schema for tool parameters (generated via schemars)

**Lifecycle**: Registered at server startup, static for foundation phase.

**Validation Rules**:
- `name` must be valid MCP tool name (alphanumeric + underscores)
- `description` must be non-empty
- `input_schema` must be valid JSON Schema draft-07

**Example** (get_server_time):
```json
{
  "name": "get_server_time",
  "description": "Returns Binance server time in milliseconds",
  "inputSchema": {
    "type": "object",
    "properties": {},
    "additionalProperties": false
  }
}
```

**Related Entities**: ToolRouter (registry)

---

### 5. ToolCall

**Purpose**: Represents an incoming MCP tool execution request.

**Attributes**:
- `name: String` - Tool name to invoke
- `arguments: Value` - Tool parameters as JSON value

**Lifecycle**: Ephemeral; created per MCP request, processed, then dropped.

**Validation Rules**:
- `name` must exist in registered tools
- `arguments` must validate against tool's input_schema

**Related Entities**: ToolResult (output), Tool (definition)

---

### 6. ToolResult

**Purpose**: Represents the result of a tool execution.

**Attributes**:
- `content: Vec<Content>` - Array of content items
- `is_error: Option<bool>` - Whether result represents an error

**Content Types**:
- Text: `{ "type": "text", "text": "..." }`
- Error: `{ "type": "text", "text": "...", "isError": true }`

**Lifecycle**: Created after tool execution, serialized to JSON-RPC response, then dropped.

**Validation Rules**:
- `content` must contain at least one item
- Text content must be valid UTF-8
- Error results must set `is_error: true`

**Example** (success):
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"serverTime\":1609459200000}"
    }
  ]
}
```

**Example** (error):
```json
{
  "content": [
    {
      "type": "text",
      "text": "Connection error: Failed to connect to Binance API"
    }
  ],
  "isError": true
}
```

**Related Entities**: ToolCall (input)

---

### 7. ServerTimeResponse

**Purpose**: Represents the response from Binance `/api/v3/time` endpoint.

**Attributes**:
- `server_time: i64` - Unix timestamp in milliseconds

**Lifecycle**: Deserialized from Binance API response, used to construct ToolResult, then dropped.

**Validation Rules**:
- `server_time` must be positive integer
- Must represent valid Unix timestamp (reasonable date range)

**API Source**: `GET https://api.binance.com/api/v3/time`

**Example**:
```json
{
  "serverTime": 1609459200000
}
```

**Related Entities**: BinanceClient (fetcher), ToolResult (consumer)

---

### 8. BinanceError

**Purpose**: Structured error from Binance API.

**Attributes**:
- `code: i32` - Binance error code (e.g., -1121, -1003)
- `msg: String` - Error message

**Lifecycle**: Deserialized from Binance error responses, converted to ToolResult with error flag.

**Common Error Codes**:
- `-1003`: Too many requests (rate limit)
- `-1021`: Timestamp outside recv_window
- `-1121`: Invalid symbol

**Example**:
```json
{
  "code": -1003,
  "msg": "Too many requests."
}
```

**Related Entities**: BinanceClient (error handling)

---

### 9. McpError

**Purpose**: Internal error type for all server failures.

**Error Variants**:
- `ConnectionError(String)` - Network failures, Binance unreachable
- `RateLimitError(String)` - 429 responses from Binance
- `ParseError(String)` - JSON deserialization failures
- `InvalidRequest(String)` - MCP protocol violations
- `NotReady(String)` - Server not fully initialized
- `InternalError(String)` - Unexpected failures

**Lifecycle**: Created on error conditions, propagated up, converted to ToolResult or JSON-RPC error.

**Validation Rules**:
- Error messages must not contain sensitive data (API keys, secrets)
- User-facing errors must be actionable

**Conversion to MCP Errors**:
```
McpError::ConnectionError → isError: true, type: "connection_error"
McpError::RateLimitError → isError: true, type: "rate_limit"
McpError::ParseError → isError: true, type: "parse_error"
```

**HTTP Status Code Mapping** (from Binance API to McpError):
```
HTTP 429 (Rate Limit)        → McpError::RateLimitError
HTTP 418 (IP Auto-Banned)    → McpError::ConnectionError (with ban context)
HTTP 5xx (Server Errors)     → McpError::ConnectionError
HTTP 4xx (Client Errors)     → McpError::InvalidRequest
HTTP 403 (WAF Limit)         → McpError::ConnectionError (with WAF context)
Network Timeout              → McpError::ConnectionError
JSON Parse Failure           → McpError::ParseError
```

**Related Entities**: All error-producing operations

---

## Entity Relationships

```
ServerInfo
└── ServerCapabilities
    └── tools: ToolsCapability

Credentials (loaded from env)
└── used by → BinanceClient

Tool (registered at startup)
└── referenced by → ToolRouter

ToolCall (incoming request)
├── validated against → Tool.input_schema
└── executed by → ToolRouter
    └── produces → ToolResult

BinanceClient
├── consumes → Credentials
├── fetches → ServerTimeResponse
└── may produce → BinanceError
    └── converted to → McpError
        └── converted to → ToolResult (error)
```

## State Management

### Server State
- **Type**: `BinanceServer` struct
- **Contents**:
  - `tool_router: ToolRouter` - Tool registry and dispatcher
  - `credentials: Option<Credentials>` - API credentials (None if not configured)
  - `http_client: reqwest::Client` - Shared HTTP client
- **Lifetime**: Entire server lifetime (startup → shutdown)
- **Concurrency**: Shared via `Clone` trait (Arc-wrapped internally)

### Request State
- **Type**: Ephemeral per-request data
- **Contents**: ToolCall, ToolResult, intermediate API responses
- **Lifetime**: Single MCP request duration (<10s)
- **Concurrency**: Isolated per request (no sharing)

## Type Safety Notes

### SecretString
- Custom newtype wrapper around `String`
- Implements `Debug` to mask value: `SecretString(***)`
- Implements `Display` with truncation: `{first4}...{last4}`
- Never derives `Serialize`

### JsonSchema Generation
- All tool parameter types must derive `schemars::JsonSchema`
- Empty structs for tools with no parameters
- Generated schemas validated at compile time via tests

## Validation Strategy

### Compile-Time Validation
- Type system enforces: non-null fields, enum exhaustiveness
- schemars ensures JSON Schema generation correctness
- thiserror ensures error type completeness

### Runtime Validation
- Env var presence checked at startup
- JSON Schema validation on incoming tool calls
- HTTP response deserialization with serde (fails fast on invalid JSON)
- Timestamp reasonableness checks (not enforced strictly but logged)

## Future Extensions (Out of Scope for Foundation)

- **AuthenticatedRequest**: Request signing with HMAC SHA256
- **OrderRequest / OrderResponse**: Trading operations
- **AccountInfo**: Balance queries
- **WebSocketMessage**: Real-time stream data
- **RateLimitState**: Weight tracking and enforcement
- **Cache**: Response caching for market data

These will be added in future features with their own data-model.md documents.
