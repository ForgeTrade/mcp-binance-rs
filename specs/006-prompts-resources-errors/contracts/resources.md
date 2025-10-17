# Resources API Contract

**Feature**: [../spec.md](../spec.md)
**Requirements**: FR-008 to FR-016
**Created**: 2025-10-17

## Contract Overview

This document defines the MCP Resources API contract for the Binance MCP Server, including resource URIs, content formats, and error handling.

---

## Resource Category 1: Market Data

**Requirement**: FR-010

**URI Pattern**: `binance://market/{symbol}`

### Resource: binance://market/btcusdt

**Description**: Real-time 24-hour ticker data for Bitcoin/USDT trading pair

**MCP Registration**:

```json
{
  "uri": "binance://market/btcusdt",
  "name": "BTCUSDT Market Data",
  "description": "Real-time 24-hour ticker statistics for Bitcoin/USDT trading pair",
  "mimeType": "text/markdown"
}
```

**Read Request**:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/read",
  "params": {
    "uri": "binance://market/btcusdt"
  }
}
```

**Success Response** (FR-013, FR-014):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "contents": [
      {
        "uri": "binance://market/btcusdt",
        "mimeType": "text/markdown",
        "text": "# BTCUSDT Market Data\n\n**Symbol**: BTCUSDT\n**Last Price**: $50,234.56\n**24h Change**: +$1,234.56 (+2.5%)\n**24h High**: $51,000.00\n**24h Low**: $49,000.00\n**24h Volume**: 12,345.67 BTC\n**Quote Volume**: $620,123,456.78 USDT\n\n**Weighted Average Price**: $50,150.32\n**Price Change Count**: 45,678\n\n*Last updated: 2025-10-17T14:23:45.123Z*\n*Data source: Binance API v3*"
      }
    ]
  }
}
```

**Acceptance Criteria** (User Story 3):

1. **Given** a user asks about Bitcoin, **When** Claude accesses the `binance://market/btcusdt` resource, **Then** Claude receives markdown-formatted market data without needing a tool call

2. **Given** the resource is accessed, **When** Claude reads the resource content, **Then** the response includes current price, 24h change, volume, and high/low values

3. **Given** multiple questions about the same symbol, **When** Claude accesses the resource repeatedly, **Then** each access returns fresh data efficiently

---

### Resource: binance://market/ethusdt

**Description**: Real-time 24-hour ticker data for Ethereum/USDT trading pair

**MCP Registration**:

```json
{
  "uri": "binance://market/ethusdt",
  "name": "ETHUSDT Market Data",
  "description": "Real-time 24-hour ticker statistics for Ethereum/USDT trading pair",
  "mimeType": "text/markdown"
}
```

**Success Response** (Example):

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "contents": [
      {
        "uri": "binance://market/ethusdt",
        "mimeType": "text/markdown",
        "text": "# ETHUSDT Market Data\n\n**Symbol**: ETHUSDT\n**Last Price**: $3,000.50\n**24h Change**: -$45.25 (-1.5%)\n**24h High**: $3,050.00\n**24h Low**: $2,980.00\n**24h Volume**: 25,678.90 ETH\n**Quote Volume**: $77,036,700.00 USDT\n\n**Weighted Average Price**: $3,005.12\n**Price Change Count**: 38,234\n\n*Last updated: 2025-10-17T14:23:50.456Z*\n*Data source: Binance API v3*"
      }
    ]
  }
}
```

---

## Resource Category 2: Account Information

**Requirement**: FR-011

**URI Pattern**: `binance://account/{type}`

### Resource: binance://account/balances

**Description**: Current account balances for all assets with non-zero values

**MCP Registration**:

```json
{
  "uri": "binance://account/balances",
  "name": "Account Balances",
  "description": "Current account balances for all assets (free and locked)",
  "mimeType": "text/markdown"
}
```

**Read Request**:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "resources/read",
  "params": {
    "uri": "binance://account/balances"
  }
}
```

**Success Response** (FR-013, FR-014):

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "contents": [
      {
        "uri": "binance://account/balances",
        "mimeType": "text/markdown",
        "text": "# Account Balances\n\n| Asset | Free | Locked | Total |\n|-------|------|--------|-------|\n| BTC | 0.50000000 | 0.00000000 | 0.50000000 |\n| ETH | 5.20000000 | 0.50000000 | 5.70000000 |\n| USDT | 10,000.00000000 | 500.00000000 | 10,500.00000000 |\n| BNB | 20.00000000 | 0.00000000 | 20.00000000 |\n\n**Total Assets**: 4\n**Trading Enabled**: Yes\n**Withdrawal Enabled**: No (Testnet)\n**Deposit Enabled**: No (Testnet)\n\n*Last updated: 2025-10-17T14:24:00.789Z*\n*Note: Testnet account - no real funds*"
      }
    ]
  }
}
```

**Acceptance Criteria** (User Story 4):

1. **Given** a user asks about their balances, **When** Claude accesses `binance://account/balances` resource, **Then** Claude receives formatted balance information

2. **Given** resource URIs are available, **When** Claude lists resources, **Then** all relevant market and account resources appear in the list

---

## Resource Category 3: Order Management

**Requirement**: FR-012

**URI Pattern**: `binance://orders/{status}`

### Resource: binance://orders/open

**Description**: All currently active orders across all trading pairs

**MCP Registration**:

```json
{
  "uri": "binance://orders/open",
  "name": "Open Orders",
  "description": "All currently active orders (NEW, PARTIALLY_FILLED)",
  "mimeType": "text/markdown"
}
```

**Read Request**:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/read",
  "params": {
    "uri": "binance://orders/open"
  }
}
```

**Success Response** (FR-013, FR-014):

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "contents": [
      {
        "uri": "binance://orders/open",
        "mimeType": "text/markdown",
        "text": "# Open Orders\n\n| Order ID | Symbol | Side | Type | Price | Orig Qty | Executed Qty | Status | Time |\n|----------|--------|------|------|-------|----------|--------------|--------|------|\n| 12345 | BTCUSDT | BUY | LIMIT | 49,000.00 | 0.001 | 0.000 | NEW | 2025-10-17 14:20:00 |\n| 12346 | ETHUSDT | SELL | LIMIT | 3,100.00 | 0.5 | 0.000 | NEW | 2025-10-17 14:22:15 |\n| 12347 | BNBUSDT | BUY | LIMIT | 295.00 | 10.0 | 5.0 | PARTIALLY_FILLED | 2025-10-17 14:18:30 |\n\n**Total Open Orders**: 3\n**Total Value**: ~$52,475.00 (estimated)\n\n*Last updated: 2025-10-17T14:24:05.234Z*\n*Note: Testnet orders*"
      }
    ]
  }
}
```

**Edge Case - No Open Orders**:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "contents": [
      {
        "uri": "binance://orders/open",
        "mimeType": "text/markdown",
        "text": "# Open Orders\n\nNo open orders found.\n\n*Last updated: 2025-10-17T14:24:10.567Z*"
      }
    ]
  }
}
```

**Acceptance Criteria** (User Story 4):

1. **Given** a user has open orders, **When** Claude accesses `binance://orders/open` resource, **Then** Claude receives list of active orders

---

## Resource List Response

**Requirement**: FR-008, FR-016

**List Request**:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "resources/list"
}
```

**Success Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "resources": [
      {
        "uri": "binance://market/btcusdt",
        "name": "BTCUSDT Market Data",
        "description": "Real-time 24-hour ticker statistics for Bitcoin/USDT trading pair",
        "mimeType": "text/markdown"
      },
      {
        "uri": "binance://market/ethusdt",
        "name": "ETHUSDT Market Data",
        "description": "Real-time 24-hour ticker statistics for Ethereum/USDT trading pair",
        "mimeType": "text/markdown"
      },
      {
        "uri": "binance://account/balances",
        "name": "Account Balances",
        "description": "Current account balances for all assets (free and locked)",
        "mimeType": "text/markdown"
      },
      {
        "uri": "binance://orders/open",
        "name": "Open Orders",
        "description": "All currently active orders (NEW, PARTIALLY_FILLED)",
        "mimeType": "text/markdown"
      }
    ]
  }
}
```

**Success Criteria**: SC-005 - Claude Desktop lists at least 5 resources (4 defined + future extensions)

---

## Error Handling

### Error: Resource Not Found (FR-015)

**Read Request** (Invalid URI):

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "resources/read",
  "params": {
    "uri": "binance://invalid/resource"
  }
}
```

**Error Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "error": {
    "code": -32404,
    "message": "Resource not found: binance://invalid/resource",
    "data": {
      "provided_uri": "binance://invalid/resource",
      "valid_categories": ["market", "account", "orders"],
      "valid_examples": [
        "binance://market/btcusdt",
        "binance://account/balances",
        "binance://orders/open"
      ],
      "recovery_suggestion": "Check URI format: binance://{category}/{identifier}"
    }
  }
}
```

### Error: Invalid Symbol (Market Resource)

**Read Request**:

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "resources/read",
  "params": {
    "uri": "binance://market/invalidsymbol"
  }
}
```

**Error Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "error": {
    "code": -32003,
    "message": "Invalid trading symbol 'INVALIDSYMBOL'. Expected format: BTCUSDT, ETHUSDT",
    "data": {
      "provided_symbol": "INVALIDSYMBOL",
      "valid_examples": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
      "recovery_suggestion": "Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT)"
    }
  }
}
```

### Error: Rate Limited

**Error Response** (429 from Binance API):

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "error": {
    "code": -32001,
    "message": "Rate limit exceeded. Please wait 60 seconds before retrying.",
    "data": {
      "retry_after_secs": 60,
      "current_weight": 1200,
      "weight_limit": 1200,
      "recovery_suggestion": "Reduce request frequency or wait for rate limit window to reset"
    }
  }
}
```

**Edge Case Handling** (from spec.md):

1. **Binance API temporarily unavailable**: Return error with retry suggestion
2. **Symbol with no recent trading activity**: Return available data with staleness indication
3. **Rate limits hit during resource read**: Return rate limit error with retry_after
4. **User has zero balances**: Return "No holdings" message instead of empty table
5. **Incorrect URI format**: Return clear 404-style error with valid URI examples

---

## Capability Registration

**Requirement**: FR-016

**Server Capabilities Response**:

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "prompts": {
      "listChanged": false
    },
    "resources": {
      "subscribe": false,
      "listChanged": false
    },
    "tools": {
      "listChanged": false
    }
  },
  "serverInfo": {
    "name": "mcp-binance-server",
    "version": "0.1.0"
  }
}
```

**Note**: `subscribe: false` indicates resources do not support real-time subscriptions (deferred to Phase 2 WebSocket implementation).

---

## Implementation Notes

### Rust Handler Signatures (FR-008, FR-009):

```rust
use rmcp::handler::server::ServerHandler;
use rmcp::model::{Resource, ResourceContents, TextContent};

impl ServerHandler for BinanceServer {
    async fn list_resources(&self) -> Result<Vec<Resource>, ErrorData> {
        Ok(vec![
            Resource {
                uri: "binance://market/btcusdt".to_string(),
                name: "BTCUSDT Market Data".to_string(),
                description: Some("Real-time 24-hour ticker statistics for Bitcoin/USDT trading pair".to_string()),
                mime_type: Some("text/markdown".to_string()),
            },
            // ... other resources
        ])
    }

    async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContents>, ErrorData> {
        let parsed_uri = ResourceUri::parse(uri)
            .map_err(|e| ErrorData::new(-32404, format!("Invalid resource URI: {}", e)))?;

        match parsed_uri.category {
            ResourceCategory::Market => self.read_market_resource(parsed_uri.identifier).await,
            ResourceCategory::Account => self.read_account_resource(parsed_uri.identifier).await,
            ResourceCategory::Orders => self.read_orders_resource(parsed_uri.identifier).await,
        }
    }
}
```

### Markdown Formatting Standards (FR-013):

- Use H1 headers for resource title (`# BTCUSDT Market Data`)
- Use bold (**) for field labels (e.g., `**Last Price**`)
- Use tables for structured data (balances, orders)
- Include timestamp at end with italic formatting (`*Last updated: ...*`)
- Format numbers consistently:
  - Prices: 2 decimal places with $ prefix
  - Crypto amounts: 8 decimal places
  - Percentages: 1-2 decimal places with % suffix
- Include data source attribution (`*Data source: Binance API v3*`)

### URI Parsing Logic (FR-010, FR-011, FR-012):

```rust
pub struct ResourceUri {
    pub scheme: String,       // "binance"
    pub category: ResourceCategory,
    pub identifier: Option<String>,
}

impl ResourceUri {
    pub fn parse(uri: &str) -> Result<Self, String> {
        // Parse: binance://{category}/{identifier}
        // category: market, account, orders
        // identifier: optional (btcusdt, balances, open)
    }
}
```

### Symbol Case Normalization:

- **User-facing URIs**: Lowercase symbols (`binance://market/btcusdt`)
- **Binance API calls**: Uppercase symbols (`BTCUSDT`)
- **Conversion**: `.to_uppercase()` before API call, `.to_lowercase()` in URI

---

## Performance Considerations

### Resource Access Efficiency (SC-003):

**Metric**: Resources reduce tool call count by 40%

**Scenario**: User asks 3 questions about Bitcoin price

**Without Resources** (Tool Calls):
1. "What's Bitcoin price?" → `get_ticker` tool call
2. "Has it changed much?" → `get_ticker` tool call
3. "Should I buy now?" → `get_ticker` tool call
**Total**: 3 tool calls

**With Resources**:
1. "What's Bitcoin price?" → Read `binance://market/btcusdt` resource
2. "Has it changed much?" → Claude uses cached resource data
3. "Should I buy now?" → Claude uses cached resource data
**Total**: 1 resource read (client-side caching)

**Result**: 66% reduction (exceeds 40% target)

### Timestamp Freshness (FR-014):

- All resources include ISO 8601 timestamp
- Timestamps sourced from Binance API response or server time
- Claude can assess data freshness for decision-making

---

## Testing Strategy

### Unit Tests:

```rust
#[tokio::test]
async fn test_list_resources() {
    let server = BinanceServer::new(/* ... */);
    let resources = server.list_resources().await.unwrap();

    assert!(resources.len() >= 4);
    assert!(resources.iter().any(|r| r.uri == "binance://market/btcusdt"));
    assert!(resources.iter().any(|r| r.uri == "binance://account/balances"));
}

#[tokio::test]
async fn test_read_market_resource() {
    let server = BinanceServer::new(/* testnet */);
    let contents = server.read_resource("binance://market/btcusdt").await.unwrap();

    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].uri, "binance://market/btcusdt");
    assert_eq!(contents[0].mime_type, Some("text/markdown".to_string()));
    assert!(contents[0].text.contains("BTCUSDT"));
}

#[tokio::test]
async fn test_read_resource_not_found() {
    let server = BinanceServer::new(/* ... */);
    let result = server.read_resource("binance://invalid/resource").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, -32404);
}
```

### Integration Tests (Testnet):

1. List resources → verify at least 4 resources returned
2. Read `binance://market/btcusdt` → verify markdown format and price data
3. Read `binance://account/balances` → verify balance table format
4. Read invalid URI → verify error code -32404 with examples
5. Read resource during rate limit → verify error code -32001

---

## Success Criteria Mapping

| Success Criterion | Contract Coverage |
|-------------------|-------------------|
| SC-003: 40% reduction in tool call count | Resource caching eliminates repeated get_ticker calls |
| SC-005: At least 5 resources listed | 4 resources defined, extensible for Phase 2-4 |
| SC-007: Complete market context | Market resources include all ticker fields formatted for LLM |

---

**Contract Status**: ✅ Complete - Ready for Implementation
