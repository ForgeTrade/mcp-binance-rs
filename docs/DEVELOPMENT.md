# Development Guide

## Building from Source

```bash
# Debug build (fast compilation, no optimizations)
cargo build

# Release build (optimized for production)
cargo build --release
```

## Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test integration
```

## Code Quality

### Pre-Commit Checks

Before committing, ensure all checks pass:

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Security audit
cargo audit

# Run all checks
cargo fmt --check && cargo clippy -- -D warnings && cargo audit
```

### Automatic Pre-Commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
echo "→ Checking code formatting..."
cargo fmt --check

# Clippy
echo "→ Running clippy..."
cargo clippy -- -D warnings

# Audit
echo "→ Running security audit..."
cargo audit

echo "✓ All pre-commit checks passed!"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Logging

The server uses `tracing` for structured logging. Control log levels via `RUST_LOG`:

```bash
# Info level (default)
RUST_LOG=info cargo run

# Debug level (shows more detail)
RUST_LOG=debug cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run

# Module-specific logging
RUST_LOG=mcp_binance_server=debug,rmcp=info cargo run
```

**Important**: Credentials are only logged at DEBUG level with `[SENSITIVE DATA]` prefix.

## Project Structure

```
mcp-provider-binance/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── main.rs             # Binary entry point
│   ├── lib.rs              # Library exports
│   ├── error.rs            # Error types
│   ├── server/             # MCP server implementation
│   │   ├── mod.rs
│   │   ├── handler.rs      # ServerHandler trait
│   │   └── tool_router.rs  # Tool routing
│   ├── tools/              # MCP tool implementations
│   │   ├── mod.rs
│   │   └── get_server_time.rs
│   ├── binance/            # Binance API client
│   │   ├── mod.rs
│   │   ├── client.rs       # HTTP client
│   │   └── types.rs        # API types
│   └── config/             # Configuration
│       ├── mod.rs
│       └── credentials.rs  # Credential loading
├── tests/
│   └── integration/        # Integration tests
└── docs/
    └── DEVELOPMENT.md      # This file
```

## Adding New Features

This project uses SpecKit for specification-driven development:

```bash
# 1. Create specification
/speckit.specify "Feature description"

# 2. Clarify ambiguities
/speckit.clarify

# 3. Generate implementation plan
/speckit.plan

# 4. Generate tasks
/speckit.tasks

# 5. Analyze for issues
/speckit.analyze

# 6. Implement
/speckit.implement
```

See `.specify/memory/constitution.md` for core principles.

## Testing Against MCP Clients

### Manual Testing with Claude Desktop

1. Build release binary: `cargo build --release`
2. Update Claude Desktop config with binary path
3. Restart Claude Desktop
4. Test MCP tools in conversation

### Testing with MCP Inspector

```bash
# Install MCP Inspector (if available)
npm install -g @modelcontextprotocol/inspector

# Run inspector against server
mcp-inspector ./target/release/mcp-binance-server
```

### JSON-RPC Manual Testing

```bash
# Run server
./target/release/mcp-binance-server

# Send initialize (in separate terminal)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"Test","version":"1.0.0"}}}' | ./target/release/mcp-binance-server
```

## Performance Profiling

```bash
# CPU profiling with flamegraph
cargo install flamegraph
sudo flamegraph -- ./target/release/mcp-binance-server

# Memory profiling with heaptrack (Linux)
heaptrack ./target/release/mcp-binance-server
```

## Troubleshooting

### Compilation Errors

**Error**: "failed to resolve: use of undeclared type"
- **Fix**: Ensure all modules are declared in parent `mod.rs`

**Error**: "cannot find rmcp in this scope"
- **Fix**: Run `cargo update` to fetch dependencies

### Runtime Errors

**Error**: "BINANCE_API_KEY environment variable not set"
- **Fix**: Export env vars or start without credentials (OK for get_server_time)

**Error**: "Connection refused"
- **Fix**: Check internet connection, Binance API availability

### Logging Issues

**Problem**: No logs appearing
- **Fix**: Set `RUST_LOG=info` or higher

**Problem**: Credentials visible in logs
- **Fix**: Only happens at DEBUG level; normal behavior

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Follow SpecKit workflow for changes
4. Ensure all pre-commit checks pass
5. Submit pull request

## Resources

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp)
- [Binance API Docs](https://binance-docs.github.io/apidocs/spot/en/)
- [Tokio Guide](https://tokio.rs/tokio/tutorial)
