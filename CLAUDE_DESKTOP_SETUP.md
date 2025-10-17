# Claude Desktop Setup Guide

## 🎯 Quick Setup

MCP Binance Server is now configured and ready to use with Claude Desktop!

### Configuration Location

```
~/Library/Application Support/Claude/claude_desktop_config.json
```

### Current Configuration

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

## 🔄 Activation Steps

1. **Restart Claude Desktop**
   - Press `Cmd+Q` to quit
   - Reopen Claude Desktop

2. **Verify Connection**
   - Look for 🔌 icon in the input field
   - Click it to see available MCP servers
   - "binance" should appear in the list

3. **Test the Tools**
   - Start a new chat
   - Try commands like: "Get current price for BTCUSDT"

## 🛠️ Available Tools

### 📊 Market Data
- `get_ticker` - 24hr ticker statistics for a symbol
- `get_order_book` - Order book depth (bids/asks)
- `get_recent_trades` - Recent trades list
- `get_klines` - Candlestick/OHLCV data
- `get_average_price` - Current average price

### 👤 Account
- `get_account_info` - Account information and balances
- `get_account_trades` - Trade history for a symbol

### 📝 Orders
- `place_order` - Place a new order (⚠️ TESTNET only!)
- `get_order` - Query order status
- `cancel_order` - Cancel an active order
- `get_open_orders` - List all open orders
- `get_all_orders` - Order history for a symbol

## 💬 Example Commands

Try these in Claude Desktop chat:

```
"What's the current price of Bitcoin?"
→ Uses: get_ticker for BTCUSDT

"Show me the order book for ETHUSDT"
→ Uses: get_order_book

"Get my account information"
→ Uses: get_account_info

"Place a test buy order for 0.001 BTC at 50000 USDT"
→ Uses: place_order (TESTNET - no real money!)

"Show my open orders"
→ Uses: get_open_orders
```

## 🐛 Troubleshooting

### Server Not Appearing

1. Check Developer Console:
   - Menu → View → Developer → Toggle Developer Tools
   - Look for `[binance]` logs in Console tab

2. Verify cargo path:
   ```bash
   which cargo
   # Should output: /Users/vi/.cargo/bin/cargo
   ```

3. Check configuration file:
   ```bash
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

### Connection Errors

1. Verify API keys are valid:
   - Visit https://testnet.binance.vision/
   - Generate new keys if needed
   - Update `claude_desktop_config.json`

2. Test server manually:
   ```bash
   cd /Users/vi/project/tradeforge/mcp-binance-rs
   cargo run --release
   ```

3. Check logs in Developer Console

### Restore Original Config

If you need to revert:

```bash
cp ~/Library/Application\ Support/Claude/claude_desktop_config.json.backup \
   ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

## ⚠️ Important Notes

- **TESTNET ONLY**: All operations use Binance Testnet (testnet.binance.vision)
- **No Real Money**: Orders and trades are simulated
- **Test Credentials**: Get free testnet keys at https://testnet.binance.vision/
- **Rate Limits**: Testnet has the same rate limits as production

## 📚 Additional Resources

- [Tests Documentation](tests/README.md) - How to run integration tests
- [MCP Protocol Docs](https://modelcontextprotocol.io/) - Learn about MCP
- [Binance API Docs](https://binance-docs.github.io/apidocs/spot/en/) - API reference

## 🎉 Success Indicators

You'll know it's working when:

✅ 🔌 icon appears in Claude Desktop input field
✅ "binance" server shows in the MCP servers list
✅ Claude can execute commands like "get current BTC price"
✅ Developer console shows `[binance]` connection logs

Enjoy trading with Claude! 🚀
