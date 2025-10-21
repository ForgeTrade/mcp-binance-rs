# 🚀 MCP Binance Server

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-green.svg)](https://modelcontextprotocol.io/)

A powerful Model Context Protocol (MCP) server that brings Binance cryptocurrency exchange capabilities to AI assistants like Claude Desktop. Trade, monitor markets, and manage your portfolio through natural conversation.

## ✨ Key Features

- 🤖 **21 AI-Ready Tools** - Market data, order book depth, advanced analytics, account info, and trading operations
- 📊 **Order Book Depth Analysis** - L1 metrics, L2 depth with progressive disclosure, real-time WebSocket streams
- 📈 **Advanced Analytics** - Volume profile (POC/VAH/VAL), order flow analysis, HFT manipulation detection, market health scoring
- 🔄 **Dual Mode** - HTTP REST API + WebSocket OR MCP stdio protocol
- ⚡ **Real-Time Data** - WebSocket streams for live price and depth updates (100ms)
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
git clone https://github.com/forgequant/mcp-provider-binance.git
cd mcp-provider-binance

# Build the MCP server (with orderbook features)
cargo build --release --features orderbook

# Verify installation
./target/release/mcp-binance-server --version
```

### Feature Flags

The server supports multiple feature flags for different deployment scenarios:

| Feature | Description | Use Case |
|---------|-------------|----------|
| `orderbook` | Order book depth tools (WebSocket streaming) | Real-time market depth analysis |
| `orderbook_analytics` | Advanced analytics (volume profile, order flow) | Professional trading insights |
| `sse` | Server-Sent Events transport | Remote HTTPS access (local testing) |
| `shuttle-runtime` | Shuttle.dev cloud deployment | Production cloud deployment |
| `http-api` | REST API + WebSocket server | Alternative to MCP stdio |

**Common Build Commands:**

```bash
# Local stdio mode (default)
cargo build --release --features orderbook

# Local SSE testing
cargo build --release --features sse,orderbook_analytics

# Shuttle deployment (use shuttle deploy instead)
shuttle deploy --features shuttle-runtime,orderbook_analytics
```

### Transport Modes

The server supports two transport modes:

1. **stdio (default)** - Local communication via standard I/O (for Claude Desktop)
2. **sse** - Remote HTTPS access via Server-Sent Events (for cloud deployment)

```bash
# Run in stdio mode (default)
cargo run --features orderbook

# Run in SSE mode
cargo run --features sse,orderbook -- --mode sse --port 8000
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
      "command": "cargo",
      "args": [
        "run",
        "--release",
        "--features",
        "orderbook",
        "--manifest-path",
        "/path/to/mcp-provider-binance/Cargo.toml"
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

## ☁️ Cloud Deployment (Shuttle.dev)

Deploy the MCP server to Shuttle.dev for remote HTTPS access from anywhere.

### Prerequisites

- [Shuttle CLI](https://docs.shuttle.dev/getting-started/installation) installed
- Shuttle account (free tier available)

### Deployment Steps

```bash
# 1. Login to Shuttle
shuttle login

# 2. Deploy with SSE transport (creates HTTPS endpoint automatically)
shuttle deploy --features shuttle-runtime,orderbook_analytics

# 3. Configure secrets (Binance API credentials)
shuttle secrets add BINANCE_API_KEY=your_testnet_api_key
shuttle secrets add BINANCE_API_SECRET=your_testnet_secret_key

# 4. Get deployment URL
shuttle status
# Returns: https://mcp-provider-binance.shuttleapp.rs

# 5. View logs
shuttle logs
```

### Claude Desktop Remote Configuration

```json
{
  "mcpServers": {
    "binance-cloud": {
      "url": "https://mcp-provider-binance.shuttleapp.rs/mcp/sse",
      "transport": "sse"
    }
  }
}
```

### Local SSE Testing

Test SSE transport locally before deploying:

```bash
# Start SSE server on localhost
cargo run --features sse,orderbook_analytics -- --mode sse --port 8000

# Test health endpoint
curl http://localhost:8000/health

# Test SSE handshake
curl -H "Accept: text/event-stream" -i http://localhost:8000/mcp/sse
```

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

### 📊 Order Book Depth Tools

#### `get_orderbook_metrics`
Get L1 market metrics with spread, mid-price, and liquidity analysis.

**Parameters**:
- `symbol` - Trading pair (e.g., "BTCUSDT", "ETHUSDT")

**Features**:
- **Compact Format**: Uses ~15% of tokens compared to full order book
- **Real-Time Data**: WebSocket streams with 100ms updates
- **Comprehensive Metrics**: Spread, mid-price, liquidity depth, walls detection

**Example**: *"Show me orderbook metrics for BTCUSDT"*

```json
Response: {
  "symbol": "BTCUSDT",
  "timestamp": 1699564800123,
  "spread_bps": 0.15,
  "mid_price": "67650.50",
  "bid_liquidity_10bps": "123.45",
  "ask_liquidity_10bps": "98.76",
  "walls": {
    "bids": [{"price": "67500.00", "qty": "50.0"}],
    "asks": []
  }
}
```

#### `get_orderbook_depth`
Get L2 order book depth with configurable levels (progressive disclosure).

**Parameters**:
- `symbol` - Trading pair
- `levels` - Number of levels (20 or 100)
  - **20 levels** (L2-lite): Uses ~50% of tokens, optimized for quick analysis
  - **100 levels** (L2-full): Uses 100% of tokens, comprehensive market depth

**Features**:
- **Compact Integer Encoding**: 40% smaller JSON than string decimals
- **Real-Time Updates**: WebSocket streams with 100ms latency
- **Progressive Disclosure**: Start with 20 levels, upgrade to 100 if needed

**Example**: *"Get orderbook depth for ETHUSDT with 20 levels"*

```json
Response: {
  "symbol": "ETHUSDT",
  "timestamp": 1699564800123,
  "price_scale": 100,
  "qty_scale": 100000,
  "bids": [
    [308500, 1050000],  // $3085.00, 10.5 ETH
    [308450, 520000]    // $3084.50, 5.2 ETH
  ],
  "asks": [
    [308600, 830000],   // $3086.00, 8.3 ETH
    [308650, 1210000]   // $3086.50, 12.1 ETH
  ]
}
```

**Decoding**:
- `price = scaled_price / price_scale` (e.g., 308500 / 100 = $3085.00)
- `quantity = scaled_qty / qty_scale` (e.g., 1050000 / 100000 = 10.5)

#### `get_orderbook_health`
Check WebSocket connection health and data freshness.

**Parameters**: None

**Status Values**:
- `ok` - All orderbook connections healthy
- `degraded` - Some connections experiencing issues
- `error` - Critical failures detected

**Example**: *"Check orderbook service health"*

```json
Response: {
  "status": "ok",
  "orderbook_symbols_active": 3,
  "last_update_age_ms": 127,
  "websocket_connected": true,
  "reason": null
}
```

### 📈 Advanced Analytics Tools

**Requires**: `orderbook_analytics` feature flag

These tools provide institutional-grade market microstructure analysis for professional traders.

#### `get_order_flow`
Analyze order flow dynamics to identify buying/selling pressure and smart money activity.

**Parameters**:
- `symbol` - Trading pair (e.g., "BTCUSDT")
- `window_duration_secs` - Analysis window in seconds (optional, default: 60)

**Outputs**:
- **Buy/Sell Volumes**: Aggressor trade volumes by side
- **Imbalance Ratio**: 0.0 (all sells) to 1.0 (all buys)
- **Net Pressure**: Cumulative buying minus selling pressure
- **Trade Counts**: Market order frequency analysis

**Use Cases**:
- Detect institutional accumulation/distribution
- Identify trend reversals via flow divergences
- Time entries during strong directional pressure

**Example**: *"Analyze order flow for BTCUSDT over last 2 minutes"*

#### `get_volume_profile`
Generate volume profile histogram to identify high-volume price nodes (support/resistance).

**Parameters**:
- `symbol` - Trading pair
- `duration_hours` - Lookback period in hours (optional, default: 24)
- `tick_size` - Price binning increment (e.g., "0.01" for BTCUSDT)

**Outputs**:
- **POC (Point of Control)**: Price level with maximum traded volume
- **VAH (Value Area High)**: Upper boundary of 70% volume concentration
- **VAL (Value Area Low)**: Lower boundary of 70% volume concentration
- **Histogram**: Volume distribution across all price levels

**Use Cases**:
- Identify key support/resistance from volume nodes
- Set targets at VAH/VAL boundaries
- Fade moves outside value area for mean reversion

**Example**: *"Generate 24-hour volume profile for ETHUSDT with 0.1 tick size"*

#### `detect_market_anomalies`
Detect HFT manipulation patterns and systemic risk indicators.

**Parameters**:
- `symbol` - Trading pair
- `window_duration_secs` - Analysis window (optional, default: 60)

**Detected Patterns**:
1. **Quote Stuffing**: >500 order updates/sec with <10% fill rate (HFT latency attack)
2. **Iceberg Orders**: Hidden institutional orders with >5x median refill frequency
3. **Flash Crash Risk**: >80% liquidity loss + >10x spread widening + >90% cancellation rate

**Outputs**:
- Anomaly type, severity (Low/Medium/High/Critical)
- Confidence score (0.0-1.0)
- Actionable recommendations (delay execution, widen stops, halt trading)

**Use Cases**:
- Avoid entering during HFT manipulation windows
- Detect hidden institutional orders (icebergs) for confluence
- Exit positions before flash crash conditions materialize

**Example**: *"Check for market anomalies in BTCUSDT"*

#### `get_liquidity_vacuums`
Identify low-liquidity price zones prone to rapid price discovery (slippage risk).

**Parameters**:
- `symbol` - Trading pair
- `window_duration_secs` - Analysis window (optional, default: 300)
- `threshold_percentile` - Detection sensitivity (optional, default: 25, range: 1-50)

**Outputs**:
- **Vacuum Zones**: Price ranges with abnormally low volume
- **Severity**: Critical (avoid entirely) / High (reduce size) / Medium (use limits)
- **Recommendations**: Stop loss placement guidance

**Use Cases**:
- Avoid placing stop losses in vacuum zones (likely to get run)
- Anticipate rapid moves through low-liquidity areas
- Size positions smaller when entering near vacuums

**Example**: *"Find liquidity vacuums in SOLUSDT"*

#### `get_microstructure_health`
Calculate composite market health score (0-100) from four microstructure components.

**Parameters**:
- `symbol` - Trading pair
- `window_duration_secs` - Analysis window (optional, default: 300)

**Components** (weighted):
- **Spread Stability** (25%): Bid-ask spread volatility
- **Liquidity Depth** (35%): Orderbook thickness across levels
- **Flow Balance** (25%): Bid/ask flow equilibrium
- **Update Rate** (15%): Market activity level

**Scoring Levels**:
- **80-100 (Excellent)**: Safe to trade aggressively with normal position sizes
- **60-79 (Good)**: Normal conditions, standard risk management
- **40-59 (Fair)**: Exercise caution, tighter stops, smaller sizes
- **20-39 (Poor)**: Reduce positions by 50%, widen stops
- **0-19 (Critical)**: HALT new trades, exit or hedge existing positions

**Use Cases**:
- Pre-trade risk assessment before entering large positions
- Dynamic position sizing based on market conditions
- Early warning system for deteriorating liquidity

**Example**: *"Check market health for BTCUSDT"*

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

### Basic Market Data
```
You: "What's the current Bitcoin price?"
Claude: [Uses get_ticker] "Bitcoin is currently trading at $50,234.56..."

You: "Show me the order book depth for ETHUSDT"
Claude: [Uses get_order_book] "Here's the ETHUSDT order book..."
```

### Order Book Analysis
```
You: "Show me orderbook metrics for BTCUSDT"
Claude: [Uses get_orderbook_metrics] "BTCUSDT orderbook metrics:
- Spread: 0.15 bps (tight spread, good liquidity)
- Mid Price: $67,650.50
- Bid liquidity within 10 bps: 123.45 BTC
- Ask liquidity within 10 bps: 98.76 BTC
- Large wall detected at $67,500 (50 BTC)"

You: "Get detailed depth with 20 levels"
Claude: [Uses get_orderbook_depth with levels=20] "Here's the L2 depth for BTCUSDT:
Top 3 bids: $67,650.00 (1.2 BTC), $67,649.50 (0.8 BTC), $67,649.00 (1.5 BTC)
Top 3 asks: $67,650.50 (0.9 BTC), $67,651.00 (1.1 BTC), $67,651.50 (0.7 BTC)"

You: "Is the orderbook service healthy?"
Claude: [Uses get_orderbook_health] "Orderbook service is healthy:
- Status: OK
- Active symbols: 3
- Last update: 127ms ago
- WebSocket: Connected"
```

### Account & Trading
```
You: "What's my account balance?"
Claude: [Uses get_account_info] "Your account has 0.5 BTC and 10,000 USDT..."

You: "Place a test buy order for 0.001 BTC at 49000"
Claude: [Uses place_order] "Order placed successfully! Order ID: 12345..."

You: "Show my open orders"
Claude: [Uses get_open_orders] "You have 3 open orders..."
```

## 🎯 Prompts Support

This server provides AI-guided prompts for trading analysis and risk assessment:

### `trading_analysis`
Get comprehensive trading analysis and recommendations for a specific trading pair.

**Parameters**:
- `symbol` - Trading pair (e.g., "BTCUSDT", "ETHUSDT")
- `strategy` - Optional: "aggressive", "balanced", or "conservative"
- `risk_tolerance` - Optional: "low", "medium", or "high"

**Example**:
```
You: "Analyze BTCUSDT for aggressive trading"
Claude: [Uses trading_analysis prompt] "Based on current market data:
- Current Price: $50,234.56 (+2.5% in 24h)
- Volume: Strong upward momentum
- Recommendation: Consider entering long positions
- Risk Factors: High volatility, monitor support at $49,000"
```

### `portfolio_risk`
Assess your portfolio risk based on current account balances and market conditions.

**Parameters**: None (uses your API credentials)

**Requires**: API credentials

**Example**:
```
You: "Assess my portfolio risk"
Claude: [Uses portfolio_risk prompt] "Portfolio Risk Analysis:
- Total Balance: $15,234.56 (0.5 BTC + 10,000 USDT)
- Risk Level: Moderate
- BTC Exposure: 33%
- Recommendations: Diversify into stable assets..."
```

### `market_microstructure_analysis` (Advanced)
Deep dive into market microstructure with volume profile, anomaly detection, and health scoring.

**Requires**: `orderbook_analytics` feature

**Parameters**:
- `symbol` - Trading pair (e.g., "BTCUSDT")
- `profile_hours` - Volume profile lookback period (optional, default: 24)

**Provides**:
- Volume profile analysis (POC/VAH/VAL levels)
- Market anomaly detection (quote stuffing, icebergs, flash crash risk)
- Microstructure health score (0-100 composite)
- Actionable trading recommendations

**Example**:
```
You: "Analyze BTCUSDT market microstructure"
Claude: [Uses market_microstructure_analysis prompt] "
# BTCUSDT Market Microstructure Analysis

## Volume Profile (Last 24h)
- POC: $67,850 (high-volume node, strong support)
- VAH: $68,200 (70% value area upper bound)
- VAL: $67,500 (70% value area lower bound)
- Current price vs POC: +0.3% (trading at fair value)

## Market Anomalies
⚠️ Quote Stuffing detected (High severity)
- Update rate: 750/sec (>500 threshold)
- Fill rate: 8% (<10% threshold)
- Action: Delay execution, widen spreads

## Market Health: 78.5 (Good)
- Spread stability: 85.2 (excellent)
- Liquidity depth: 92.1 (excellent)
- Flow balance: 68.3 (good)
- Update rate: 72.0 (good)

Recommendation: Normal trading conditions with caution around HFT activity windows."
```

### `order_flow_trading` (Advanced)
Order flow-based trading signals with institutional pressure analysis.

**Requires**: `orderbook_analytics` feature

**Parameters**:
- `symbol` - Trading pair
- `window_minutes` - Flow analysis window in minutes (optional, default: 5)

**Provides**:
- Real-time buy/sell pressure analysis
- Imbalance ratio and net pressure metrics
- Smart money detection (large order flow)
- Entry/exit signal recommendations

**Example**:
```
You: "Get order flow signals for ETHUSDT"
Claude: [Uses order_flow_trading prompt] "
# ETHUSDT Order Flow Analysis (Last 5 min)

## Flow Metrics
- Buy volume: 245.3 ETH
- Sell volume: 198.7 ETH
- Imbalance ratio: 0.552 (moderate bullish pressure)
- Net pressure: +46.6 ETH

## Signal: LONG BIAS ✅
- Aggressor buys dominating (55% of volume)
- Institutional accumulation detected (3 large buy orders)
- Recommendation: Look for entries on minor pullbacks
- Stop loss: Below recent swing low ($3,045)
- Target: VAH level ($3,086)"
```

### `liquidity_mapping` (Advanced)
Comprehensive liquidity landscape with vacuum zones and wall identification.

**Requires**: `orderbook_analytics` feature

**Parameters**:
- `symbol` - Trading pair
- `lookback_minutes` - Historical analysis window (optional, default: 30)

**Provides**:
- Liquidity vacuum zones (low-volume price gaps)
- Large order walls (bid/ask support/resistance)
- Optimal stop loss placement zones
- Slippage risk assessment

**Example**:
```
You: "Map liquidity for SOLUSDT"
Claude: [Uses liquidity_mapping prompt] "
# SOLUSDT Liquidity Map

## Liquidity Vacuums 🕳️
1. $145.50 - $148.20 (High severity)
   - Avg volume: 12.5 SOL (85% below median)
   - Risk: Rapid price discovery, avoid stops here

2. $142.00 - $143.50 (Medium severity)
   - Avg volume: 34.8 SOL (55% below median)

## Order Walls 🧱
- Bid wall: $144.00 (250 SOL) - Strong support
- Ask wall: $150.00 (310 SOL) - Resistance zone

## Stop Loss Guidance
- Safe zones: Below $143.80 or above $148.50 (outside vacuums)
- Risky zones: $145-$148 (vacuum zone, likely to get run)

Current price: $146.75 (inside vacuum - expect volatility)"
```

## 📦 Resources Support

Access live market data and account information through MCP resources:

### Market Resources
- `binance://market/btcusdt` - Real-time BTCUSDT market data (price, volume, 24h stats)
- `binance://market/ethusdt` - Real-time ETHUSDT market data

Returns markdown-formatted ticker data with current price, 24h change, volume, and high/low prices.

**Example**:
```
You: "Show me the BTCUSDT market resource"
Claude: [Reads binance://market/btcusdt] "# BTCUSDT Market Data
Current Price: $50,234.56
24h Change: +$1,234.56 (+2.5%)
Volume: 12,345.67 BTC
High: $51,000.00 | Low: $49,000.00"
```

### Account Resources
- `binance://account/balances` - Your current account balances (all assets with non-zero balance)

**Requires**: API credentials

Returns markdown table with Asset, Free Balance, Locked Balance, and Total columns.

**Example**:
```
You: "Show my account balances resource"
Claude: [Reads binance://account/balances] "# Account Balances

| Asset | Free Balance | Locked Balance | Total |
|-------|--------------|----------------|-------|
| BTC   | 0.50000000   | 0.00000000     | 0.50000000 |
| USDT  | 10000.00     | 500.00         | 10500.00 |"
```

### Orders Resources
- `binance://orders/open` - Your currently active orders

**Requires**: API credentials

Returns markdown table with Order ID, Symbol, Side, Type, Price, Quantity, and Status columns.

**Example**:
```
You: "Show my open orders resource"
Claude: [Reads binance://orders/open] "# Open Orders

| Order ID | Symbol  | Side | Type  | Price     | Quantity | Status |
|----------|---------|------|-------|-----------|----------|--------|
| 12345    | BTCUSDT | BUY  | LIMIT | 49000.00  | 0.001    | NEW    |"
```

## ⚠️ Enhanced Error Handling

The server provides detailed error messages with recovery suggestions:

| Error Code | Category | Description | Recovery Actions |
|------------|----------|-------------|------------------|
| `-32001` | Rate Limit | Too many requests | Wait for retry_after seconds, reduce frequency |
| `-32002` | Authentication | Invalid/missing API credentials | Check BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables |
| `-32003` | Validation | Invalid parameters (symbol, quantity, etc.) | Review parameter format and examples |
| `-32004` | Trading | Insufficient balance or trading restrictions | Check account balance and trading permissions |

**Example Error Response**:
```json
{
  "code": -32001,
  "message": "Rate limit exceeded. Please wait 60 seconds before retrying.",
  "data": {
    "retry_after_secs": 60,
    "current_weight": 1200,
    "weight_limit": 1200,
    "recovery_suggestion": "Reduce request frequency or wait for rate limit window to reset"
  }
}
```

All errors include:
- **Clear message**: Human-readable description
- **Error code**: Standard MCP error code for programmatic handling
- **Recovery data**: Specific actions to resolve the issue (retry timing, missing config, format examples)

## 🔧 Advanced Usage

### HTTP REST API Mode

Run as a standalone HTTP server with WebSocket support:

```bash
# Build with HTTP + orderbook features
cargo build --release --features http-api,websocket,orderbook

# Configure
export HTTP_BEARER_TOKEN="your_secure_token"
export HTTP_HOST="0.0.0.0"
export HTTP_PORT="3000"
export BINANCE_API_KEY="your_key"
export BINANCE_SECRET_KEY="your_secret"

# Start server
./target/release/mcp-binance-server
```

### Feature Flags

Control which features to compile:

```bash
# Core features only (13 tools)
cargo build --release

# Add orderbook depth tools (16 tools)
cargo build --release --features orderbook

# Add advanced analytics tools (21 tools total = 16 + 5 analytics)
cargo build --release --features orderbook_analytics

# Add HTTP API + WebSocket
cargo build --release --features http-api,websocket

# All features
cargo build --release --features orderbook_analytics,http-api,websocket
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

1. **Check config path**: Ensure `cargo` is in your PATH
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
"command": "/path/to/.cargo/bin/cargo"
```

Or add cargo to your PATH and restart Claude Desktop.

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

# Run tests with orderbook features
cargo test --features orderbook

# Run integration tests
cargo test --test '*' --features orderbook

# Run performance tests
cargo test --features orderbook --release -- --nocapture test_l1_metrics_performance
cargo test --features orderbook --release -- --nocapture test_l2_depth_performance

# Run with logging
RUST_LOG=debug cargo test --features orderbook

# Test specific tool
cargo test test_get_ticker
cargo test test_orderbook_metrics --features orderbook
```

**Test Coverage**:
- **44 automated tests** for orderbook features (100% pass rate)
- Unit tests: OrderBook types, manager, errors (23 tests)
- Integration tests: WebSocket reconnection, rate limiting, metrics (16 tests)
- Performance tests: L1 metrics (<200ms P95), L2 depth (<300ms P95) (5 tests)

## 🔧 Troubleshooting

### Common Issues

#### SSE Connection Errors

**Problem**: `503 Service Unavailable` when connecting to SSE endpoint

**Solutions**:
- Server has reached 50 concurrent connection limit - wait for other connections to close
- Check server logs: `shuttle logs` (cloud) or check terminal output (local)
- Restart server to clear stale connections

**Problem**: `404 Not Found` on `/mcp/message` endpoint

**Solutions**:
- Connection ID expired or invalid
- Re-establish connection via `/mcp/sse` handshake
- Check `X-Connection-ID` header is being sent correctly

**Problem**: Connection timeout after 30 seconds

**Solutions**:
- SSE sessions automatically timeout after 30s of inactivity
- Implement keepalive pings from client
- Re-establish connection when needed

#### Shuttle Deployment Issues

**Problem**: `secrets not found` error on deployment

**Solutions**:
```bash
# Add required secrets
shuttle secrets add BINANCE_API_KEY=your_testnet_key
shuttle secrets add BINANCE_API_SECRET=your_testnet_secret

# Verify secrets are set
shuttle secrets list
```

**Problem**: Deployment fails to build

**Solutions**:
- Ensure Rust 1.90+ installed: `rustc --version`
- Clear cache: `cargo clean`
- Retry deployment: `shuttle deploy --features shuttle-runtime,orderbook_analytics`

**Problem**: Server returns `WARN: No API credentials configured`

**Solutions**:
- This is expected if no secrets are set
- Authenticated features (trading, account info) will be disabled
- Market data tools will still work without credentials

#### Rate Limiting

**Problem**: `429 Too Many Requests` from Binance API

**Solutions**:
- Server has built-in GCRA rate limiter (1000 req/min)
- Wait 60 seconds and retry
- Reduce frequency of tool calls

**Problem**: Server queue timeout after 30s

**Solutions**:
- Too many concurrent requests in queue
- Reduce parallelism in your queries
- Wait for previous requests to complete

#### Build Errors

**Problem**: `feature 'orderbook_analytics' requires feature 'orderbook'`

**Solutions**:
```bash
# Always include base feature when using analytics
cargo build --features orderbook_analytics  # ❌ Wrong
cargo build --features orderbook,orderbook_analytics  # ✅ Correct
```

**Problem**: RocksDB lock contention in tests

**Solutions**:
```bash
# Run tests sequentially (orderbook_analytics uses shared RocksDB)
cargo test --features orderbook_analytics -- --test-threads=1
```

### Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/forgequant/mcp-provider-binance/issues)
- **Logs**: Enable debug logging with `RUST_LOG=debug` environment variable
- **MCP Protocol**: See [Model Context Protocol docs](https://modelcontextprotocol.io/)
- **Shuttle Support**: [Shuttle.dev Discord](https://discord.gg/shuttle)

### Deployment Checklist

Before deploying to production:

- [ ] Binance API credentials obtained (use TESTNET for testing)
- [ ] Shuttle CLI installed and authenticated (`shuttle login`)
- [ ] Secrets configured (`shuttle secrets add BINANCE_API_KEY=...`)
- [ ] Local testing completed (`cargo run --features sse -- --mode sse`)
- [ ] Health endpoint verified (`curl https://your-app.shuttleapp.rs/health`)
- [ ] SSE handshake tested (`curl -H "Accept: text/event-stream" https://your-app.shuttleapp.rs/mcp/sse`)
- [ ] Claude Desktop configured with remote URL
- [ ] Logs monitored after deployment (`shuttle logs`)

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

- **GitHub**: [forgequant/mcp-provider-binance](https://github.com/forgequant/mcp-provider-binance)
- **Binance Testnet**: [testnet.binance.vision](https://testnet.binance.vision/)
- **MCP Protocol**: [modelcontextprotocol.io](https://modelcontextprotocol.io/)
- **Issues**: [GitHub Issues](https://github.com/forgequant/mcp-provider-binance/issues)

---

Made with ❤️ using Rust and MCP | [Report Bug](https://github.com/forgequant/mcp-provider-binance/issues) | [Request Feature](https://github.com/forgequant/mcp-provider-binance/issues)
