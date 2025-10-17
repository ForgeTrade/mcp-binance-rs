# Data Model: MCP Enhancements - Prompts, Resources & Error Handling

**Feature**: [spec.md](spec.md)
**Created**: 2025-10-17
**Status**: Phase 1 Design

## Entity Definitions

### Entity 1: Prompt Definition

**Source**: spec.md Key Entities

**Description**: Represents a registered AI prompt with name, description, arguments schema, and handler function. Contains metadata for MCP protocol and parameters for data fetching.

**Rust Implementation**:

```rust
// Defined by rmcp macros, but conceptually:
pub struct PromptDefinition {
    pub name: String,                    // e.g., "trading_analysis"
    pub description: String,             // Human-readable purpose
    pub arguments_schema: serde_json::Value, // JSON Schema from schemars
    // Handler function registered via #[prompt] attribute
}

// Arguments for trading_analysis prompt (FR-002)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TradingAnalysisArgs {
    #[schemars(description = "Trading pair symbol (e.g., BTCUSDT, ETHUSDT)")]
    pub symbol: String,

    #[schemars(description = "Trading strategy preference")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<TradingStrategy>,

    #[schemars(description = "Risk tolerance level")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_tolerance: Option<RiskTolerance>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TradingStrategy {
    Aggressive,
    Balanced,
    Conservative,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RiskTolerance {
    Low,
    Medium,
    High,
}

// Arguments for portfolio_risk prompt (FR-004)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioRiskArgs {
    // Empty struct - no parameters required
    // Account info is derived from API credentials
}
```

**MCP Protocol Representation**:

```json
{
  "name": "trading_analysis",
  "description": "Analyze market conditions for a specific cryptocurrency and provide trading recommendations",
  "arguments": [
    {
      "name": "symbol",
      "description": "Trading pair symbol (e.g., BTCUSDT, ETHUSDT)",
      "required": true
    },
    {
      "name": "strategy",
      "description": "Trading strategy preference: aggressive, balanced, or conservative",
      "required": false
    },
    {
      "name": "risk_tolerance",
      "description": "Risk tolerance level: low, medium, or high",
      "required": false
    }
  ]
}
```

**Relationships**:
- Contains → TradingAnalysisArgs (parameters)
- Invokes → BinanceClient API methods (data fetching)
- Returns → GetPromptResult (MCP response)

**Validation Rules**:
- `symbol` must be uppercase alphanumeric (validated by Binance API)
- `strategy` enum limited to 3 values
- `risk_tolerance` enum limited to 3 values
- Optional parameters default to None

**Constitution Compliance**:
- ✅ Core Principle IV: Type-safe via schemars JSON Schema
- ✅ Core Principle V: MCP-compliant GetPromptResult return type

---

### Entity 2: Resource Definition

**Source**: spec.md Key Entities

**Description**: Represents an accessible data endpoint with URI, name/description, and content format. Includes logic for fetching and formatting real-time data.

**Rust Implementation**:

```rust
// Defined by rmcp, but conceptually:
pub struct ResourceDefinition {
    pub uri: String,           // e.g., "binance://market/btcusdt"
    pub name: String,          // Display name
    pub description: String,   // Human-readable purpose
    pub mime_type: String,     // "text/markdown"
}

// Resource URI parser
pub struct ResourceUri {
    pub scheme: String,       // "binance"
    pub category: ResourceCategory,
    pub identifier: Option<String>, // e.g., "btcusdt" for market resources
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceCategory {
    Market,    // Market data resources
    Account,   // Account information resources
    Orders,    // Order management resources
}

impl ResourceUri {
    /// Parse hierarchical URI: binance://{category}/{identifier}
    pub fn parse(uri: &str) -> Result<Self, String> {
        let parts: Vec<&str> = uri.split("://").collect();
        if parts.len() != 2 || parts[0] != "binance" {
            return Err(format!("Invalid scheme. Expected 'binance://', got '{}'", uri));
        }

        let path_parts: Vec<&str> = parts[1].split('/').collect();
        if path_parts.is_empty() {
            return Err("Missing resource category".to_string());
        }

        let category = match path_parts[0] {
            "market" => ResourceCategory::Market,
            "account" => ResourceCategory::Account,
            "orders" => ResourceCategory::Orders,
            other => return Err(format!("Unknown category: {}", other)),
        };

        let identifier = if path_parts.len() > 1 {
            Some(path_parts[1].to_string())
        } else {
            None
        };

        Ok(ResourceUri {
            scheme: "binance".to_string(),
            category,
            identifier,
        })
    }
}
```

**MCP Protocol Representation**:

```json
{
  "uri": "binance://market/btcusdt",
  "name": "BTCUSDT Market Data",
  "description": "Real-time 24-hour ticker data for Bitcoin/USDT trading pair",
  "mimeType": "text/markdown"
}
```

**Resource Content Format** (Markdown):

```markdown
# BTCUSDT Market Data

**Last Price**: $50,234.56
**24h Change**: +2.5% ($1,234.56)
**24h High**: $51,000.00
**24h Low**: $49,000.00
**24h Volume**: 12,345.67 BTC

*Last updated: 2025-10-17 14:23:45 UTC*
```

**Relationships**:
- Contains → ResourceUri (identifier)
- Fetches → BinanceClient API data
- Returns → Markdown-formatted content string

**Validation Rules**:
- URI scheme must be "binance"
- Category must be one of: market, account, orders
- Market resources require symbol identifier (lowercase)
- Account/orders resources may omit identifier
- Invalid URIs return ResourceNotFound error (FR-015)

**Constitution Compliance**:
- ✅ Core Principle IV: Type-safe ResourceCategory enum
- ✅ Core Principle V: MCP Resource type with mime_type
- ✅ Security § Compliance: No sensitive identifiers in URIs

---

### Entity 3: Error Context

**Source**: spec.md Key Entities

**Description**: Represents enriched error information including error type, user message, recovery suggestions, and structured debug data for troubleshooting.

**Rust Implementation**:

```rust
use thiserror::Error;
use serde_json::json;
use std::time::Duration;

/// Enhanced error types for Phase 1 (FR-017 to FR-024)
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimited {
        retry_after: Duration,
        current_weight: u32,
        weight_limit: u32,
    },

    #[error("Invalid API credentials. Check environment variables")]
    InvalidCredentials {
        masked_key: String,      // First 4 + last 4 characters only
        help_url: String,        // Link to testnet docs
    },

    #[error("Invalid trading symbol: {provided}")]
    InvalidSymbol {
        provided: String,
        format_help: String,     // "Expected format: BTCUSDT, ETHUSDT"
        examples: Vec<String>,   // ["BTCUSDT", "ETHUSDT", "BNBUSDT"]
    },

    #[error("Insufficient {asset} balance")]
    InsufficientBalance {
        asset: String,
        required: String,        // Decimal string
        available: String,       // Decimal string
    },

    /// Wrapper for existing API errors (backward compatibility)
    #[error("Binance API error: {0}")]
    ApiError(#[from] reqwest::Error),
}

impl From<BinanceError> for rmcp::model::ErrorData {
    fn from(err: BinanceError) -> Self {
        use rmcp::model::ErrorCode;

        match err {
            BinanceError::RateLimited { retry_after, current_weight, weight_limit } => {
                ErrorData::new(
                    ErrorCode::Custom(-32001),
                    format!("Rate limit exceeded. Please wait {} seconds before retrying.", retry_after.as_secs())
                )
                .with_data(json!({
                    "retry_after_secs": retry_after.as_secs(),
                    "current_weight": current_weight,
                    "weight_limit": weight_limit,
                    "recovery_suggestion": "Reduce request frequency or wait for rate limit window to reset"
                }))
            },

            BinanceError::InvalidCredentials { masked_key, help_url } => {
                ErrorData::new(
                    ErrorCode::Custom(-32002),
                    "Invalid API credentials. Please check your BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables."
                )
                .with_data(json!({
                    "masked_api_key": masked_key,
                    "help_url": help_url,
                    "recovery_suggestion": "Verify credentials at https://testnet.binance.vision/ and ensure correct environment variables"
                }))
            },

            BinanceError::InvalidSymbol { provided, format_help, examples } => {
                ErrorData::new(
                    ErrorCode::Custom(-32003),
                    format!("Invalid trading symbol '{}'. {}", provided, format_help)
                )
                .with_data(json!({
                    "provided_symbol": provided,
                    "valid_examples": examples,
                    "recovery_suggestion": "Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT)"
                }))
            },

            BinanceError::InsufficientBalance { asset, required, available } => {
                ErrorData::new(
                    ErrorCode::Custom(-32004),
                    format!("Insufficient {} balance. Required: {}, Available: {}", asset, required, available)
                )
                .with_data(json!({
                    "asset": asset,
                    "required_amount": required,
                    "available_amount": available,
                    "recovery_suggestion": "Deposit more funds or reduce order quantity"
                }))
            },

            BinanceError::ApiError(e) => {
                ErrorData::new(
                    ErrorCode::InternalError,
                    format!("Binance API error: {}", e)
                )
            },
        }
    }
}

/// Helper function to mask API keys for error messages (FR-019)
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    format!("{}****{}", &key[..4], &key[key.len()-4..])
}
```

**MCP Protocol Representation** (Rate Limit Error Example):

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

**Relationships**:
- Converts → rmcp::model::ErrorData (MCP protocol)
- Contains → Structured recovery suggestions (JSON)
- Sanitizes → Sensitive data (masked API keys, no stack traces)

**Validation Rules**:
- Masked API keys show only first 4 + last 4 characters
- Error codes in -32000 to -32099 range (custom MCP errors)
- Recovery suggestions must be actionable and user-friendly
- No internal paths, stack traces, or secrets in messages

**Constitution Compliance**:
- ✅ Core Principle I: Error messages sanitized, no API keys exposed
- ✅ Core Principle IV: Type-safe error variants with structured data
- ✅ Security § Data Protection: Sensitive data not in logs (DEBUG only)

---

### Entity 4: Trading Analysis Arguments

**Source**: spec.md Key Entities

**Description**: Parameters for trading analysis prompt including symbol (required), strategy preference (optional: aggressive/balanced/conservative), and risk tolerance (optional: low/medium/high).

**Rust Implementation** (Covered in Entity 1, extracted here for clarity):

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TradingAnalysisArgs {
    /// Trading pair symbol (e.g., BTCUSDT, ETHUSDT)
    pub symbol: String,

    /// Optional trading strategy preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<TradingStrategy>,

    /// Optional risk tolerance level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_tolerance: Option<RiskTolerance>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TradingStrategy {
    Aggressive,   // High-frequency, short-term trades
    Balanced,     // Mixed approach
    Conservative, // Low-risk, long-term holds
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RiskTolerance {
    Low,     // Risk-averse, prefer stable assets
    Medium,  // Moderate risk acceptable
    High,    // High-risk, high-reward tolerance
}

impl Default for TradingAnalysisArgs {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            strategy: None,
            risk_tolerance: None,
        }
    }
}
```

**Usage in Prompt Handler**:

```rust
#[prompt(name = "trading_analysis", description = "Analyze market conditions and provide trading recommendations")]
async fn trading_analysis(
    &self,
    Parameters(args): Parameters<TradingAnalysisArgs>,
    _ctx: RequestContext<RoleServer>,
) -> Result<GetPromptResult, ErrorData> {
    // Fetch ticker data
    let ticker = self.binance_client
        .get_24hr_ticker(&args.symbol)
        .await
        .map_err(|e| BinanceError::from(e))?;

    // Format market data with strategy context
    let mut content = format!(
        "# Market Analysis: {}\n\n\
        **Current Price**: ${}\n\
        **24h Change**: {}% ({})\n\
        **24h High**: ${}\n\
        **24h Low**: ${}\n\
        **24h Volume**: {} {}\n\n",
        ticker.symbol,
        ticker.last_price,
        ticker.price_change_percent,
        ticker.price_change,
        ticker.high_price,
        ticker.low_price,
        ticker.volume,
        ticker.symbol.trim_end_matches("USDT") // Extract base asset
    );

    // Add strategy context if provided
    if let Some(strategy) = args.strategy {
        content.push_str(&format!("**Strategy Preference**: {:?}\n", strategy));
    }

    if let Some(risk) = args.risk_tolerance {
        content.push_str(&format!("**Risk Tolerance**: {:?}\n", risk));
    }

    Ok(GetPromptResult {
        messages: vec![PromptMessage {
            role: Role::User,
            content: TextContent::text(content),
        }]
    })
}
```

**Relationships**:
- Deserialized from → MCP prompt invocation JSON
- Validated by → schemars JSON Schema
- Used by → trading_analysis prompt handler
- Fetches → BinanceClient::get_24hr_ticker data

**Validation Rules**:
- `symbol` required, non-empty string
- `strategy` optional, limited to 3 enum values
- `risk_tolerance` optional, limited to 3 enum values
- Invalid enums rejected at deserialization (serde validation)

---

### Entity 5: Resource URI

**Source**: spec.md Key Entities

**Description**: Hierarchical identifier for resources following pattern `binance://{category}/{identifier}` where category is "market", "account", or "orders".

**Rust Implementation** (Covered in Entity 2, design rationale extracted here):

**Design Rationale** (from research.md Decision 3):

```
Hierarchical URI Pattern: binance://{category}/{identifier}

Categories:
  - market/    : Real-time market data (ticker, order book)
  - account/   : Account information (balances, positions)
  - orders/    : Order management (open orders, order history)

Examples:
  binance://market/btcusdt          → 24hr ticker for BTCUSDT (FR-010)
  binance://market/ethusdt          → 24hr ticker for ETHUSDT
  binance://account/balances        → All account balances (FR-011)
  binance://orders/open             → All open orders (FR-012)

Future Extensions (Phase 2-4):
  binance://market/btcusdt/depth    → Order book depth
  binance://orders/open/btcusdt     → Symbol-specific open orders
  binance://history/trades/btcusdt  → Historical trades
  binance://futures/usds/btcusdt    → Futures market data (Phase 4)
```

**Parser Implementation**:

```rust
pub struct ResourceUri {
    pub scheme: String,                 // Always "binance"
    pub category: ResourceCategory,     // Parsed from first path segment
    pub identifier: Option<String>,     // Parsed from second path segment
}

impl ResourceUri {
    pub fn to_string(&self) -> String {
        let category_str = match self.category {
            ResourceCategory::Market => "market",
            ResourceCategory::Account => "account",
            ResourceCategory::Orders => "orders",
        };

        if let Some(ref id) = self.identifier {
            format!("binance://{}/{}", category_str, id)
        } else {
            format!("binance://{}", category_str)
        }
    }
}
```

**Validation Rules**:
- Scheme MUST be "binance" (case-sensitive)
- Category MUST be one of: "market", "account", "orders"
- Identifier optional, lowercase alphanumeric for symbols
- Symbol identifiers normalized to lowercase (BTCUSDT → btcusdt in URI)
- Trailing slashes ignored during parsing

**Error Handling**:
```rust
match ResourceUri::parse(uri) {
    Ok(parsed) => { /* handle resource */ },
    Err(msg) => {
        return Err(ErrorData::new(
            ErrorCode::Custom(-32404),
            format!("Invalid resource URI: {}", msg)
        )
        .with_data(json!({
            "provided_uri": uri,
            "valid_examples": [
                "binance://market/btcusdt",
                "binance://account/balances",
                "binance://orders/open"
            ]
        })))
    }
}
```

---

## Data Flow Diagrams

### Flow 1: Prompt Invocation (trading_analysis)

```
User Query: "Should I buy Bitcoin now?"
    ↓
Claude Desktop → MCP Client
    ↓
[Prompt Invocation]
{
  "method": "prompts/get",
  "params": {
    "name": "trading_analysis",
    "arguments": { "symbol": "BTCUSDT", "strategy": "balanced" }
  }
}
    ↓
rmcp prompt_router → BinanceServer::trading_analysis()
    ↓
Parameters<TradingAnalysisArgs> deserialization (schemars validation)
    ↓
BinanceClient::get_24hr_ticker("BTCUSDT")
    ↓
[Binance API] GET /api/v3/ticker/24hr?symbol=BTCUSDT
    ↓
Ticker24hrResponse { lastPrice: "50234.56", priceChangePercent: "2.5", ... }
    ↓
Format as markdown GetPromptResult
    ↓
[MCP Response]
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "# Market Analysis: BTCUSDT\n\n**Current Price**: $50,234.56\n..."
      }
    }
  ]
}
    ↓
Claude Desktop → LLM Analysis → User Response
```

---

### Flow 2: Resource Read (market data)

```
User Query: "What's the Bitcoin price?"
    ↓
Claude Desktop → MCP Client
    ↓
[Resource Read]
{
  "method": "resources/read",
  "params": { "uri": "binance://market/btcusdt" }
}
    ↓
BinanceServer::read_resource("binance://market/btcusdt")
    ↓
ResourceUri::parse() → { category: Market, identifier: "btcusdt" }
    ↓
Match category → Market handler
    ↓
BinanceClient::get_24hr_ticker("BTCUSDT") // Uppercase for API
    ↓
[Binance API] GET /api/v3/ticker/24hr?symbol=BTCUSDT
    ↓
Ticker24hrResponse → Format as markdown
    ↓
[MCP Response]
{
  "contents": [
    {
      "uri": "binance://market/btcusdt",
      "mimeType": "text/markdown",
      "text": "# BTCUSDT Market Data\n\n**Last Price**: $50,234.56\n..."
    }
  ]
}
    ↓
Claude Desktop → Display formatted data → User Response
```

---

### Flow 3: Error Handling (Rate Limit)

```
Prompt/Resource Request
    ↓
BinanceClient::get_24hr_ticker()
    ↓
[Binance API] Returns 429 Too Many Requests
    ↓
reqwest::Error → BinanceError::RateLimited {
  retry_after: Duration::from_secs(60),
  current_weight: 1200,
  weight_limit: 1200
}
    ↓
From<BinanceError> for ErrorData
    ↓
[MCP Error Response]
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
    ↓
Claude Desktop → User-friendly error with recovery suggestion
```

---

## Entity Relationships Summary

```
┌──────────────────┐
│ PromptDefinition │ (trading_analysis, portfolio_risk)
└─────────┬────────┘
          │ uses
          ↓
┌──────────────────────┐
│ TradingAnalysisArgs  │ (symbol, strategy, risk_tolerance)
└─────────┬────────────┘
          │ validated by
          ↓
┌──────────────────┐
│ schemars Schema  │ (JSON Schema)
└──────────────────┘

┌──────────────────┐
│ ResourceURI      │ (binance://market/btcusdt)
└─────────┬────────┘
          │ parsed into
          ↓
┌──────────────────┐
│ ResourceCategory │ (Market, Account, Orders)
└─────────┬────────┘
          │ determines
          ↓
┌──────────────────┐
│ Handler Function │ (fetch_market_data, fetch_balances, etc.)
└──────────────────┘

┌──────────────────┐
│ BinanceError     │ (RateLimited, InvalidCredentials, etc.)
└─────────┬────────┘
          │ converts to
          ↓
┌──────────────────┐
│ ErrorData (MCP)  │ (code, message, recovery data)
└──────────────────┘
```

---

## Constitution Compliance Summary

| Entity | Core Principles Met | Notes |
|--------|---------------------|-------|
| Prompt Definition | II (Auto-Gen), IV (Type Safety), V (MCP) | Macros generate handlers, schemars provides schemas |
| Resource Definition | III (Modular), IV (Type Safety), V (MCP) | Hierarchical URIs support future modules |
| Error Context | I (Security), IV (Type Safety) | Sanitizes sensitive data, typed variants |
| Trading Analysis Args | IV (Type Safety), V (MCP) | schemars JSON Schema validation |
| Resource URI | IV (Type Safety), Security § Compliance | Parser prevents injection, no secrets in URIs |

---

**Phase 1 Status**: ✅ Complete - Ready for Contract Generation
