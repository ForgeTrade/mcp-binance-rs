# Claude Desktop Setup Guide

This guide walks you through integrating the MCP Binance Server with Claude Desktop.

## Prerequisites

- Claude Desktop app installed
- MCP Binance Server compiled: `cargo build --release`
- (Optional) Binance API credentials

## Configuration File Location

Claude Desktop reads MCP server configuration from:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

## Step-by-Step Setup

### 1. Build the Server

```bash
cd /path/to/mcp-binance-rs
cargo build --release
```

Verify the binary exists:
```bash
ls -lh target/release/mcp-binance-server
```

### 2. Get Absolute Path to Binary

```bash
# macOS/Linux
realpath target/release/mcp-binance-server

# Or use pwd
cd target/release && pwd
# Result: /Users/vi/project/tradeforge/mcp-binance-rs/target/release
```

**Important**: Claude Desktop requires an **absolute path**, not relative paths like `./target/release/...`

### 3. Create/Edit Configuration File

Open (or create) the `claude_desktop_config.json` file:

```bash
# macOS
open -e ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Or create if missing
mkdir -p ~/Library/Application\ Support/Claude
touch ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### 4. Add MCP Server Configuration

**Minimal Configuration** (no credentials):

```json
{
  "mcpServers": {
    "binance": {
      "command": "/absolute/path/to/mcp-binance-server"
    }
  }
}
```

**Full Configuration** (with credentials and logging):

```json
{
  "mcpServers": {
    "binance": {
      "command": "/Users/vi/project/tradeforge/mcp-binance-rs/target/release/mcp-binance-server",
      "env": {
        "BINANCE_API_KEY": "your_actual_api_key_here",
        "BINANCE_SECRET_KEY": "your_actual_secret_key_here",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Configuration with Multiple MCP Servers**:

```json
{
  "mcpServers": {
    "binance": {
      "command": "/path/to/mcp-binance-server",
      "env": {
        "BINANCE_API_KEY": "your_key",
        "BINANCE_SECRET_KEY": "your_secret"
      }
    },
    "other-server": {
      "command": "/path/to/other-mcp-server"
    }
  }
}
```

### 5. Restart Claude Desktop

**macOS**:
```bash
# Quit Claude completely
osascript -e 'quit app "Claude"'

# Wait a moment
sleep 2

# Relaunch
open -a Claude
```

**Windows**: Close Claude Desktop via system tray, then relaunch

**Linux**: `killall claude && claude &`

### 6. Verify Integration

In Claude Desktop chat, try:

```
"What time is it on the Binance servers?"
```

or

```
"Use the get_server_time tool"
```

Claude should:
1. Discover the `get_server_time` tool
2. Call the tool automatically
3. Display the current Binance server time

## Configuration Options

### Environment Variables

| Variable | Required | Description | Default |
|----------|----------|-------------|---------|
| `BINANCE_API_KEY` | No | Binance API key | None (warning logged) |
| `BINANCE_SECRET_KEY` | No | Binance secret key | None (warning logged) |
| `RUST_LOG` | No | Logging level | `info` |

### Logging Levels

- `error`: Only errors
- `warn`: Warnings + errors
- `info`: General info (recommended)
- `debug`: Detailed debug info (includes credential loading)
- `trace`: Very verbose (not recommended)

**Example with debug logging**:
```json
{
  "mcpServers": {
    "binance": {
      "command": "/path/to/mcp-binance-server",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

## Troubleshooting

### Server Not Appearing in Claude

**Check 1: Verify Configuration File Syntax**
```bash
# Validate JSON syntax
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | python3 -m json.tool
```

If this fails, you have invalid JSON. Common issues:
- Missing commas between sections
- Trailing commas (not allowed in JSON)
- Unescaped backslashes in paths (Windows)

**Check 2: Verify Binary Path**
```bash
# Test the path from config
/absolute/path/from/config --help
```

If "command not found", the path is wrong.

**Check 3: Check Binary Permissions**
```bash
ls -l /path/to/mcp-binance-server
# Should show: -rwxr-xr-x (executable)

# If not executable:
chmod +x /path/to/mcp-binance-server
```

### Tools Not Showing in Claude

**Check: View Claude Desktop Logs**

macOS:
```bash
tail -f ~/Library/Logs/Claude/mcp*.log
```

Windows:
```
%LOCALAPPDATA%\Claude\logs\mcp*.log
```

Look for:
- Connection errors
- Binary launch failures
- Protocol errors

**Check: Test Server Manually**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"Test","version":"1.0.0"}}}' | /path/to/mcp-binance-server
```

Should return JSON response with server info.

### Credentials Not Loading

**Symptom**: Logs show "No API credentials configured"

**Fix 1**: Set environment variables in `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "binance": {
      "command": "/path/to/mcp-binance-server",
      "env": {
        "BINANCE_API_KEY": "your_key_here",
        "BINANCE_SECRET_KEY": "your_secret_here"
      }
    }
  }
}
```

**Fix 2**: Check for whitespace:
```json
// WRONG (has spaces)
"BINANCE_API_KEY": " your_key "

// CORRECT
"BINANCE_API_KEY": "your_key"
```

**Fix 3**: Verify environment variables are set:
```bash
# Add debug logging
"RUST_LOG": "debug"
```

Then check logs for:
```
DEBUG mcp_binance_server::config: [SENSITIVE DATA] Loading API key: your_key
```

### Server Crashes or Restarts

**Check Logs**:
```bash
# macOS
Console.app → Search for "Claude"

# Or terminal
tail -f ~/Library/Logs/Claude/mcp*.log
```

Common issues:
- Network connectivity problems
- Invalid Binance API responses
- Rate limiting (429 errors)

## Getting Binance API Credentials

1. Go to [Binance](https://www.binance.com/)
2. Log in → Account → API Management
3. Create API Key
4. **Important**: For testing, use **Read Only** permissions
5. Copy API Key and Secret Key
6. **Never share or commit these keys**

## Security Best Practices

1. **Use Read-Only API Keys** for testing
2. **Never commit** `claude_desktop_config.json` with credentials
3. **Restrict IP Access** in Binance API settings (optional)
4. **Enable API Key Restrictions** (trading disabled for read-only)
5. **Monitor API Usage** in Binance dashboard

## Advanced: Multiple Environments

You can set up different configurations for development vs production:

**Development** (test keys):
```bash
cp ~/Library/Application\ Support/Claude/claude_desktop_config.json ~/claude_desktop_config.dev.json

# Edit dev config with test credentials
```

**Production** (real keys):
```bash
# Keep original config secure
```

Switch by replacing the config file and restarting Claude.

## Next Steps

Once integrated:
1. Try asking Claude about Binance server time
2. Explore other tools (as they're added)
3. Check logs if anything breaks
4. Report issues on GitHub

## Additional Resources

- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/)
- [Claude Desktop Documentation](https://claude.ai/docs)
- [Binance API Documentation](https://binance-docs.github.io/apidocs/)
- [Project README](../README.md)
