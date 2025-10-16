# MCP Binance Server

A Model Context Protocol (MCP) server for Binance API integration, enabling AI assistants to interact with the Binance exchange.

## Features

- **MCP Protocol Compliance**: Full implementation of MCP lifecycle (initialize → tools → execution)
- **Secure Credential Management**: API keys loaded from environment variables, never logged
- **Binance API Integration**: Currently supports `get_server_time` tool for connectivity validation
- **Async-First**: Built on Tokio for high-performance concurrent operations
- **Type-Safe**: Rust type system ensures correctness and prevents entire classes of bugs

## Prerequisites

- Rust 1.90 or later (Edition 2024)
- Internet connection for Binance API access
- Binance API credentials (optional for `get_server_time`)

## Installation

```bash
# Clone the repository
cd /Users/vi/project/tradeforge/mcp-binance-rs

# Build the server
cargo build --release

# Verify installation
./target/release/mcp-binance-server --version
```

## Configuration

The server loads Binance API credentials from environment variables:

```bash
export BINANCE_API_KEY="your_api_key_here"
export BINANCE_SECRET_KEY="your_secret_key_here"
```

**Note**: Credentials are optional for foundation features like `get_server_time`. The server will start without them and log a warning.

## Usage

### Standalone Mode

```bash
# Run with default logging (INFO)
./target/release/mcp-binance-server

# Run with debug logging
RUST_LOG=debug ./target/release/mcp-binance-server
```

### Claude Desktop Integration

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "binance": {
      "command": "/Users/vi/project/tradeforge/mcp-binance-rs/target/release/mcp-binance-server",
      "env": {
        "BINANCE_API_KEY": "your_api_key_here",
        "BINANCE_SECRET_KEY": "your_secret_key_here",
        "RUST_LOG": "info"
      }
    }
  }
}
```

Then restart Claude Desktop and ask: "What time is it on the Binance servers?"

## Available Tools

### get_server_time

Returns the current Binance server time in milliseconds (Unix timestamp).

**Parameters**: None

**Example Response**:
```json
{
  "serverTime": 1609459200000
}
```

## Troubleshooting

### Server won't start
- **Check Rust version**: Run `rustc --version` - must be 1.90+
- **Missing dependencies**: Run `cargo build` to install all dependencies
- **Port conflicts**: MCP uses stdio, no ports needed

### "No API credentials configured" warning
- This is normal if you haven't set `BINANCE_API_KEY` or `BINANCE_SECRET_KEY`
- The server will still work for tools that don't require authentication
- To add credentials, set the environment variables before starting the server

### Tools not appearing in Claude Desktop
- Verify the `command` path in `claude_desktop_config.json` is absolute and correct
- Check Claude Desktop logs: `~/Library/Logs/Claude/mcp*.log` (macOS)
- Ensure the binary is executable: `chmod +x target/release/mcp-binance-server`
- Restart Claude Desktop after configuration changes

### Rate limit errors
- Binance API has rate limits - the server automatically retries with exponential backoff
- Check logs for "rate limit" messages
- If persistent, wait 1-2 minutes before retrying

### Time offset warnings
- Large time offset (>5s) may cause API signature issues for authenticated endpoints
- Sync your system clock: macOS automatic time sync in System Preferences
- Check: `ntpdate -q time.apple.com`

## Development

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for development instructions, testing, and contribution guidelines.

## Architecture

- **src/server**: MCP protocol implementation and ServerHandler
- **src/tools**: MCP tool implementations (get_server_time, etc.)
- **src/binance**: Binance API HTTP client
- **src/config**: Configuration and credential management
- **src/error**: Error types and conversions

## Security

- ✅ API keys never stored in code or logs (INFO/WARN levels)
- ✅ Rate limiting with exponential backoff
- ✅ HTTPS enforced via rustls-tls
- ✅ Input validation before API calls
- ✅ Secrets masked in error messages

See `.specify/memory/constitution.md` for full security principles.

## License

MIT

## Contributing

This project follows a specification-driven development workflow using SpecKit. All features are added via `/speckit.specify` command.

See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for details.
