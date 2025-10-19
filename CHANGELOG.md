# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **BREAKING**: Consolidated to Streamable HTTP transport (MCP March 2025 spec)
  - Removed legacy SSE GET handshake endpoints: `/sse`, `/mcp/sse`
  - Removed non-standard endpoint: `/mcp/message`
  - All MCP communication now uses POST `/mcp` with `Mcp-Session-Id` header
  - Removed `X-Connection-ID` header validation (replaced with `Mcp-Session-Id`)
  - Removed `sse_handshake` function (~47 lines of code eliminated)

### Migration Guide

#### For existing clients using legacy SSE transport:

**Old pattern (deprecated)**:
```bash
# 1. GET handshake
curl -X GET https://server/sse
# Returns X-Connection-ID: <uuid>

# 2. POST messages with X-Connection-ID
curl -X POST https://server/messages \
  -H "X-Connection-ID: <uuid>" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
```

**New pattern (Streamable HTTP)**:
```bash
# 1. POST initialize (creates session)
curl -X POST https://server/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"client","version":"1.0"}},"id":1}'
# Returns Mcp-Session-Id: <uuid> header

# 2. POST subsequent requests with Mcp-Session-Id
curl -X POST https://server/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <uuid>" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
```

#### Backward compatibility:

- POST `/messages` endpoint is preserved as an alias to POST `/mcp` for temporary compatibility
- No changes to tool names, arguments, or response formats
- Session management behavior unchanged (50 concurrent limit, 5s timeout)

### Removed
- GET `/sse` endpoint (use POST `/mcp` with `initialize` method instead)
- GET `/mcp/sse` endpoint (custom path, non-standard)
- POST `/mcp/message` endpoint (use POST `/mcp` instead)
- `sse_handshake` function from handlers_simple.rs
- `X-Connection-ID` header support (use `Mcp-Session-Id` instead)

### Added
- Consolidated Streamable HTTP transport using single POST `/mcp` endpoint
- `Mcp-Session-Id` header for session management (MCP March 2025 spec)
- Updated server info endpoint to reflect `transport: "streamable-http"`

## [0.1.0] - 2025-10-17

### Added
- Initial MCP server implementation for Binance API
- Order book depth tools (L1 metrics, L2 depth, health monitoring)
- WebSocket streaming for real-time order book data
- ChatGPT-compatible search and fetch tools
- Shuttle.dev cloud deployment support
- SSE transport with session management

[Unreleased]: https://github.com/user/mcp-binance-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/user/mcp-binance-rs/releases/tag/v0.1.0
