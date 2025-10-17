# 🚀 MCP Binance Server

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-green.svg)](https://modelcontextprotocol.io/)

A powerful Model Context Protocol (MCP) server that brings Binance cryptocurrency exchange capabilities to AI assistants like Claude Desktop. Trade, monitor markets, and manage your portfolio through natural conversation.

## ✨ Key Features

- 🤖 **13 AI-Ready Tools** - Complete market data, account info, and trading operations
- 🔄 **Dual Mode** - HTTP REST API + WebSocket OR MCP stdio protocol
- ⚡ **Real-Time Data** - WebSocket streams for live price updates
- 🔐 **Secure** - API keys from environment, never logged
- 🎯 **Type-Safe** - Rust guarantees correctness at compile time
- 🚦 **Rate Limiting** - Automatic throttling and exponential backoff
- 📊 **TESTNET Ready** - Safe testing with Binance testnet

## 📋 Prerequisites

- **Rust** 1.90 or later (Edition 2024)
- **Binance API Credentials** - Get free testnet keys at [testnet.binance.vision](https://testnet.binance.vision/)
- **Claude Desktop** (optional) - For AI assistant integration

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
cd /Users/vi/project/tradeforge/mcp-binance-rs

# Build the MCP server
cargo build --release

# Verify installation
./target/release/mcp-binance-server --version
```

### Claude Desktop Setup

1. **Get Binance Testnet Credentials**:
   - Visit [testnet.binance.vision](https://testnet.binance.vision/)
   - Create an account and generate API keys
   - ⚠️ **Use TESTNET only!** Never use production keys

2. **Configure Claude Desktop**:

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "binance": {
      "command": "/Users/vi/.cargo/bin/cargo",
      "args": [
        "run",
        "--release",
        "--manifest-path",
        "/Users/vi/project/tradeforge/mcp-binance-rs/Cargo.toml"
      ],
      "env": {
        "BINANCE_API_KEY": "your_testnet_api_key",
        "BINANCE_SECRET_KEY": "your_testnet_secret_key",
        "BINANCE_BASE_URL": "https://testnet.binance.vision",
        "RUST_LOG": "info"
      }
    }
  }
}
```

3. **Restart Claude Desktop** (Cmd+Q, then reopen)

4. **Verify Connection**:
   - Look for 🔌 icon in Claude Desktop
   - Click it to see "binance" server
   - Try: *"What's the current price of Bitcoin?"*

## 🛠️ Available Tools

### 📊 Market Data Tools

#### `get_server_time`
Get current Binance server time for synchronization.

**Example**: *"Check Binance server time"*

```json
Response: {
  "serverTime": 1699564800000,
  "offset": -125
}
```

#### `get_ticker`
Get 24-hour price statistics for any trading pair.

**Parameters**:
- `symbol` - Trading pair (e.g., "BTCUSDT", "ETHUSDT")

**Example**: *"What's the 24hr price change for Bitcoin?"*

```json
Response: {
  "symbol": "BTCUSDT",
  "priceChange": "1234.56",
  "priceChangePercent": "2.5",
  "lastPrice": "50000.00",
  "volume": "12345.67",
  "highPrice": "51000.00",
  "lowPrice": "49000.00"
}
```

#### `get_order_book`
Get current order book with bids and asks.

**Parameters**:
- `symbol` - Trading pair
- `limit` - Depth (5, 10, 20, 50, 100, 500, 1000, 5000), default: 100

**Example**: *"Show me the order book for ETHUSDT with top 10 levels"*

```json
Response: {
  "bids": [["3000.00", "10.5"], ["2999.50", "5.2"]],
  "asks": [["3001.00", "8.3"], ["3001.50", "12.1"]]
}
```

#### `get_recent_trades`
Get recent public trades for a symbol.

**Parameters**:
- `symbol` - Trading pair
- `limit` - Number of trades (default: 500, max: 1000)

**Example**: *"Show me the last 10 trades for BTCUSDT"*

#### `get_klines`
Get candlestick/OHLCV data for technical analysis.

**Parameters**:
- `symbol` - Trading pair
- `interval` - Time period: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
- `limit` - Number of klines (default: 500, max: 1000)

**Example**: *"Get hourly candlestick data for Bitcoin"*

#### `get_average_price`
Get current average price (simpler than ticker).

**Parameters**:
- `symbol` - Trading pair

**Example**: *"What's the average price of Ethereum?"*

### 👤 Account Tools

#### `get_account_info`
Get your account information, balances, and permissions.

**Requires**: API credentials

**Example**: *"Show my account balances"*

```json
Response: {
  "balances": [
    {"asset": "BTC", "free": "0.5", "locked": "0.0"},
    {"asset": "USDT", "free": "10000.00", "locked": "500.00"}
  ],
  "canTrade": true,
  "canWithdraw": false,
  "canDeposit": false
}
```

#### `get_account_trades`
Get your personal trade history for a symbol.

**Parameters**:
- `symbol` - Trading pair
- `limit` - Number of trades (default: 500, max: 1000)

**Requires**: API credentials

**Example**: *"Show my last 20 trades on BTCUSDT"*

### 📝 Order Management Tools

#### `place_order`
Create a new trading order (BUY/SELL, LIMIT/MARKET).

⚠️ **TESTNET ONLY!** Always use testnet credentials.

**Parameters**:
- `symbol` - Trading pair
- `side` - "BUY" or "SELL"
- `type` - "LIMIT" or "MARKET"
- `quantity` - Amount to trade (e.g., "0.001")
- `price` - Price for LIMIT orders (optional for MARKET)

**Requires**: API credentials

**Example**: *"Place a limit buy order for 0.001 BTC at 50000 USDT"*

```json
Response: {
  "orderId": 12345,
  "symbol": "BTCUSDT",
  "status": "NEW",
  "side": "BUY",
  "type": "LIMIT",
  "price": "50000.00",
  "origQty": "0.001"
}
```

#### `get_order`
Query the status of a specific order.

**Parameters**:
- `symbol` - Trading pair
- `order_id` - Order ID to check

**Requires**: API credentials

**Example**: *"Check status of order 12345 for BTCUSDT"*

#### `cancel_order`
Cancel an active order.

**Parameters**:
- `symbol` - Trading pair
- `order_id` - Order ID to cancel

**Requires**: API credentials

**Example**: *"Cancel order 12345"*

#### `get_open_orders`
List all currently active orders.

**Parameters**:
- `symbol` - Trading pair (optional, returns all if omitted)

**Requires**: API credentials

**Example**: *"Show all my open orders"*

#### `get_all_orders`
Get complete order history (active, canceled, filled).

**Parameters**:
- `symbol` - Trading pair
- `limit` - Number of orders (default: 500, max: 1000)

**Requires**: API credentials

**Example**: *"Show my last 50 orders on ETHUSDT"*

## 💬 Example Conversations

```
You: "What's the current Bitcoin price?"
Claude: [Uses get_ticker] "Bitcoin is currently trading at $50,234.56..."

You: "Show me the order book depth for ETHUSDT"
Claude: [Uses get_order_book] "Here's the ETHUSDT order book..."

You: "What's my account balance?"
Claude: [Uses get_account_info] "Your account has 0.5 BTC and 10,000 USDT..."

You: "Place a test buy order for 0.001 BTC at 49000"
Claude: [Uses place_order] "Order placed successfully! Order ID: 12345..."

You: "Show my open orders"
Claude: [Uses get_open_orders] "You have 3 open orders..."
```

## 🔧 Advanced Usage

### HTTP REST API Mode

Run as a standalone HTTP server with WebSocket support:

```bash
# Build with HTTP features
cargo build --release --features http-api,websocket

# Configure
export HTTP_BEARER_TOKEN="your_secure_token"
export HTTP_HOST="0.0.0.0"
export HTTP_PORT="3000"
export BINANCE_API_KEY="your_key"
export BINANCE_SECRET_KEY="your_secret"

# Start server
./target/release/mcp-binance-server
```

**REST API Endpoints**: See [CLAUDE_DESKTOP_SETUP.md](CLAUDE_DESKTOP_SETUP.md) for complete API documentation.

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `BINANCE_API_KEY` | For auth | - | Binance API key |
| `BINANCE_SECRET_KEY` | For auth | - | Binance secret key |
| `BINANCE_BASE_URL` | No | production | Use `https://testnet.binance.vision` for testnet |
| `RUST_LOG` | No | `info` | Logging level: trace, debug, info, warn, error |
| `HTTP_BEARER_TOKEN` | HTTP mode | - | Authentication token for HTTP API |
| `HTTP_HOST` | No | `127.0.0.1` | HTTP server bind address |
| `HTTP_PORT` | No | `8080` | HTTP server port |

## 🐛 Troubleshooting

### Tools not appearing in Claude Desktop

1. **Check config path**: Ensure `/Users/vi/.cargo/bin/cargo` exists
   ```bash
   which cargo
   ```

2. **View logs**:
   ```bash
   tail -f ~/Library/Logs/Claude/mcp-server-binance.log
   ```

3. **Verify build**:
   ```bash
   cargo run --release
   # Should show: "Starting MCP Binance Server v0.1.0"
   ```

4. **Check Developer Console**: Claude Desktop → View → Developer → Toggle Developer Tools

### "spawn cargo ENOENT" error

Use full path to cargo in config:
```json
"command": "/Users/vi/.cargo/bin/cargo"
```

### Empty tools list `{"tools":[]}`

This was a bug that's been fixed! Make sure you have the latest version:
```bash
git pull
cargo build --release
```

### Rate limit errors

Binance has rate limits. The server automatically retries with exponential backoff:
- Wait 1-2 minutes if you hit limits
- Check logs for "rate limit" messages
- Consider reducing request frequency

### Time offset warnings

```
Large time offset detected: 6000ms. Consider syncing system clock.
```

**Solution**: Sync your system clock:
```bash
# macOS
sudo sntp -sS time.apple.com

# Linux
sudo ntpdate -u pool.ntp.org
```

## 🔒 Security Best Practices

✅ **DO**:
- Use Binance testnet for development and testing
- Store API keys in environment variables
- Use separate API keys for each application
- Enable IP whitelist on Binance (if supported)
- Set appropriate API key permissions (trading, reading)

❌ **DON'T**:
- Commit API keys to Git
- Use production keys for testing
- Share API keys
- Log secret keys
- Disable SSL/TLS verification

## 📚 Documentation

- [Claude Desktop Setup Guide](CLAUDE_DESKTOP_SETUP.md) - Detailed setup instructions
- [HTTP API Documentation](specs/003-specify-scripts-bash/quickstart.md) - REST API reference
- [Development Guide](docs/DEVELOPMENT.md) - Contributing and architecture
- [MCP Protocol](https://modelcontextprotocol.io/) - Protocol specification
- [Binance API Docs](https://binance-docs.github.io/apidocs/spot/en/) - Official API reference

## 🏗️ Architecture

```
src/
├── server/         # MCP protocol implementation
│   ├── mod.rs      # BinanceServer struct
│   ├── handler.rs  # ServerHandler implementation
│   └── tool_router.rs  # Tool routing and implementations
├── binance/        # Binance API client
│   ├── client.rs   # HTTP client
│   └── types.rs    # API response types
├── config/         # Configuration management
├── error.rs        # Error handling
└── main.rs         # Entry point
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test '*'

# Run with logging
RUST_LOG=debug cargo test

# Test specific tool
cargo test test_get_ticker
```

## 🤝 Contributing

This project uses [SpecKit](https://specify.tools/) for specification-driven development:

1. **Specify** features using `/speckit.specify`
2. **Plan** implementation with `/speckit.plan`
3. **Generate** tasks with `/speckit.tasks`
4. **Implement** with `/speckit.implement`

See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for details.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io/) - Protocol specification
- [rmcp](https://crates.io/crates/rmcp) - MCP server SDK for Rust
- [Binance](https://www.binance.com/) - Cryptocurrency exchange API
- [Anthropic Claude](https://www.anthropic.com/) - AI assistant integration

## 🔗 Links

- **GitHub**: [tradeforge/mcp-binance-rs](https://github.com/tradeforge/mcp-binance-rs)
- **Binance Testnet**: [testnet.binance.vision](https://testnet.binance.vision/)
- **MCP Protocol**: [modelcontextprotocol.io](https://modelcontextprotocol.io/)
- **Issues**: [GitHub Issues](https://github.com/tradeforge/mcp-binance-rs/issues)

---

Made with ❤️ using Rust and MCP | [Report Bug](https://github.com/tradeforge/mcp-binance-rs/issues) | [Request Feature](https://github.com/tradeforge/mcp-binance-rs/issues)
