# Quickstart: MCP Enhancements Implementation

**Feature**: [spec.md](spec.md)
**Branch**: `006-prompts-resources-errors`
**Created**: 2025-10-17

## Overview

This quickstart guide provides step-by-step instructions for implementing MCP Prompts, Resources, and Enhanced Error Handling. Follow these phases sequentially, verifying each phase before moving to the next.

---

## Prerequisites

1. **Development Environment**:
   ```bash
   cd /Users/vi/project/tradeforge/mcp-binance-rs
   git checkout 006-prompts-resources-errors
   cargo --version  # Should be 1.90+ (Edition 2024)
   ```

2. **Binance Testnet Credentials**:
   ```bash
   export BINANCE_API_KEY="your_testnet_key"
   export BINANCE_SECRET_KEY="your_testnet_secret"
   export BINANCE_BASE_URL="https://testnet.binance.vision"
   ```

3. **Reference Files**:
   - `rust-sdk/examples/servers/src/common/counter.rs` - Prompt router example
   - `docs/IMPROVEMENTS.md` - Implementation guidance
   - `specs/006-prompts-resources-errors/contracts/` - API contracts

---

## Phase 1a: Prompts Foundation (MVP)

**Goal**: Implement `trading_analysis` prompt + basic error enhancement

**Files to Modify**:
- `src/server/handler.rs` - Add prompt_handler macro
- `src/error.rs` - Create BinanceError enum
- `src/binance/types.rs` (if needed) - Add TradingAnalysisArgs

### Step 1: Create Error Types

**File**: `src/error.rs`

Add BinanceError enum with RateLimited variant:

```rust
use thiserror::Error;
use std::time::Duration;
use serde_json::json;
use rmcp::model::ErrorData;

/// Enhanced error types for Phase 1
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimited {
        retry_after: Duration,
        current_weight: u32,
        weight_limit: u32,
    },

    /// Wrapper for existing API errors
    #[error("Binance API error: {0}")]
    ApiError(#[from] reqwest::Error),
}

impl From<BinanceError> for ErrorData {
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
            BinanceError::ApiError(e) => {
                ErrorData::new(
                    ErrorCode::InternalError,
                    format!("Binance API error: {}", e)
                )
            },
        }
    }
}
```

**Verify**: `cargo build` should compile successfully.

---

### Step 2: Create Prompt Parameter Types

**File**: `src/server/types.rs` (create if doesn't exist)

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Arguments for trading_analysis prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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
    Aggressive,
    Balanced,
    Conservative,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RiskTolerance {
    Low,
    Medium,
    High,
}
```

**Verify**: `cargo build` should compile successfully.

---

### Step 3: Add Prompt Handler

**File**: `src/server/handler.rs`

Replace `#[tool_handler]` with combined macro:

```rust
use rmcp::{
    handler::server::ServerHandler,
    model::{
        GetPromptResult, PromptMessage, Role, TextContent,
        RequestContext, RoleServer, ErrorData,
    },
    tool_handler, prompt_handler, Parameters,
};
use crate::server::types::{TradingAnalysisArgs, TradingStrategy, RiskTolerance};
use crate::error::BinanceError;

// Combined tool_handler and prompt_handler
#[tool_handler(router = self.tool_router)]
#[prompt_handler]
impl ServerHandler for BinanceServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                prompts: Some(PromptsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "mcp-binance-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("Binance MCP Server".to_string()),
                website_url: Some("https://github.com/tradeforge/mcp-binance-rs".to_string()),
                icons: None,
            },
            instructions: Some(
                "Binance MCP Server for trading and market data. \
                Provides tools for market data/account/trading operations, \
                prompts for AI-guided analysis, and resources for efficient data access."
                    .to_string(),
            ),
        }
    }

    #[prompt(
        name = "trading_analysis",
        description = "Analyze market conditions for a specific cryptocurrency and provide trading recommendations"
    )]
    async fn trading_analysis(
        &self,
        Parameters(args): Parameters<TradingAnalysisArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        // Fetch 24hr ticker data
        let ticker = self.binance_client
            .get_24hr_ticker(&args.symbol)
            .await
            .map_err(|e| {
                // Convert reqwest::Error to BinanceError
                BinanceError::ApiError(e).into()
            })?;

        // Format market data as markdown
        let mut content = format!(
            "# Market Analysis: {}\n\n\
            **Symbol**: {}\n\
            **Last Price**: ${}\n\
            **24h Change**: {} ({}%)\n\
            **24h High**: ${}\n\
            **24h Low**: ${}\n\
            **24h Volume**: {} {}\n\
            **Quote Volume**: ${} USDT\n\n",
            ticker.symbol,
            ticker.symbol,
            ticker.last_price,
            ticker.price_change,
            ticker.price_change_percent,
            ticker.high_price,
            ticker.low_price,
            ticker.volume,
            ticker.symbol.trim_end_matches("USDT"),
            ticker.quote_volume,
        );

        // Add strategy context if provided
        if let Some(strategy) = args.strategy {
            content.push_str(&format!("**Strategy Preference**: {:?}\n", strategy));
        }

        if let Some(risk) = args.risk_tolerance {
            content.push_str(&format!("**Risk Tolerance**: {:?}\n", risk));
        }

        content.push_str(&format!(
            "\n*Last updated: {}*\n\
            *Data source: Binance API v3*",
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(GetPromptResult {
            messages: vec![PromptMessage {
                role: Role::User,
                content: TextContent::text(content),
            }],
        })
    }
}
```

**Verify**:
```bash
cargo build
# Should compile successfully
```

---

### Step 4: Test Phase 1a

**Manual Test (Claude Desktop)**:

1. Start server:
   ```bash
   cargo run --release
   ```

2. In Claude Desktop, ask: "Analyze BTCUSDT for a balanced trading strategy"

3. Expected: Claude invokes `trading_analysis` prompt and displays formatted market data with analysis.

**Integration Test**:

Create `tests/test_prompts.rs`:

```rust
#[tokio::test]
async fn test_trading_analysis_prompt() {
    let server = BinanceServer::new(/* testnet config */);
    let args = TradingAnalysisArgs {
        symbol: "BTCUSDT".to_string(),
        strategy: Some(TradingStrategy::Balanced),
        risk_tolerance: Some(RiskTolerance::Medium),
    };

    let result = server.trading_analysis(Parameters(args), ctx).await;
    assert!(result.is_ok());

    let prompt_result = result.unwrap();
    assert_eq!(prompt_result.messages.len(), 1);
    assert!(prompt_result.messages[0].content.as_text().contains("BTCUSDT"));
    assert!(prompt_result.messages[0].content.as_text().contains("Balanced"));
}
```

**Run Tests**:
```bash
cargo test test_trading_analysis_prompt
```

**Success Criteria**: User Story 1 acceptance scenarios pass.

---

## Phase 1b: Portfolio Risk + Complete Errors

**Goal**: Add `portfolio_risk` prompt + remaining error types

### Step 1: Add Remaining Error Variants

**File**: `src/error.rs`

Add to `BinanceError` enum:

```rust
#[derive(Debug, Error)]
pub enum BinanceError {
    // ... existing RateLimited variant

    #[error("Invalid API credentials. Check environment variables")]
    InvalidCredentials {
        masked_key: String,
        help_url: String,
    },

    #[error("Invalid trading symbol: {provided}")]
    InvalidSymbol {
        provided: String,
        format_help: String,
        examples: Vec<String>,
    },

    #[error("Insufficient {asset} balance")]
    InsufficientBalance {
        asset: String,
        required: String,
        available: String,
    },

    // ... existing ApiError variant
}

// Add helper function
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    format!("{}****{}", &key[..4], &key[key.len()-4..])
}

// Update From<BinanceError> for ErrorData with new variants
impl From<BinanceError> for ErrorData {
    fn from(err: BinanceError) -> Self {
        use rmcp::model::ErrorCode;

        match err {
            // ... existing RateLimited case

            BinanceError::InvalidCredentials { masked_key, help_url } => {
                ErrorData::new(
                    ErrorCode::Custom(-32002),
                    "Invalid API credentials. Please check your BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables."
                )
                .with_data(json!({
                    "masked_api_key": masked_key,
                    "help_url": help_url,
                    "recovery_suggestion": "Verify credentials at https://testnet.binance.vision/"
                }))
            },

            BinanceError::InvalidSymbol { provided, format_help, examples } => {
                ErrorData::new(
                    ErrorCode::Custom(-32003),
                    format!("Invalid trading symbol '{}'. {}", provided, format_help)
                )
                .with_data(json!({
                    "provided_symbol": provided,
                    "format_help": format_help,
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

            // ... existing ApiError case
        }
    }
}
```

---

### Step 2: Add Portfolio Risk Prompt

**File**: `src/server/types.rs`

```rust
/// Arguments for portfolio_risk prompt (empty - no parameters)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioRiskArgs {}
```

**File**: `src/server/handler.rs`

Add prompt handler:

```rust
#[prompt(
    name = "portfolio_risk",
    description = "Assess portfolio risk and provide diversification recommendations based on current holdings"
)]
async fn portfolio_risk(
    &self,
    Parameters(_args): Parameters<PortfolioRiskArgs>,
    _ctx: RequestContext<RoleServer>,
) -> Result<GetPromptResult, ErrorData> {
    // Fetch account information
    let account = self.binance_client
        .get_account_info()
        .await
        .map_err(|e| BinanceError::ApiError(e).into())?;

    // Filter non-zero balances
    let balances: Vec<_> = account.balances.iter()
        .filter(|b| b.free.parse::<f64>().unwrap_or(0.0) > 0.0 || b.locked.parse::<f64>().unwrap_or(0.0) > 0.0)
        .collect();

    let mut content = String::from("# Portfolio Risk Assessment\n\n## Current Holdings\n\n");

    if balances.is_empty() {
        content.push_str("No active balances found in your account.\n\n");
        content.push_str("**Recommendation**: Deposit funds to begin trading. Start with:\n");
        content.push_str("- 30% stablecoins (USDT/BUSD) for liquidity\n");
        content.push_str("- 40% major cryptocurrencies (BTC/ETH)\n");
        content.push_str("- 30% diversified altcoins based on risk tolerance\n");
    } else {
        content.push_str("| Asset | Free Balance | Locked Balance | Total |\n");
        content.push_str("|-------|--------------|----------------|-------|\n");

        for balance in &balances {
            content.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                balance.asset,
                balance.free,
                balance.locked,
                (balance.free.parse::<f64>().unwrap_or(0.0) + balance.locked.parse::<f64>().unwrap_or(0.0))
            ));
        }

        content.push_str(&format!("\n**Total Assets**: {}\n", balances.len()));
    }

    content.push_str(&format!(
        "\n*Last updated: {}*\n*Note: Testnet account - no real funds*",
        chrono::Utc::now().to_rfc3339()
    ));

    Ok(GetPromptResult {
        messages: vec![PromptMessage {
            role: Role::User,
            content: TextContent::text(content),
        }],
    })
}
```

---

### Step 3: Test Phase 1b

**Manual Test**:

1. In Claude Desktop, ask: "What's my portfolio risk?"

2. Expected: Claude invokes `portfolio_risk` and displays balance table with risk assessment.

**Integration Test**:

```rust
#[tokio::test]
async fn test_portfolio_risk_prompt() {
    let server = BinanceServer::new(/* testnet */);
    let args = PortfolioRiskArgs {};

    let result = server.portfolio_risk(Parameters(args), ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_credentials_error() {
    let server = BinanceServer::new(/* invalid credentials */);
    let args = PortfolioRiskArgs {};

    let result = server.portfolio_risk(Parameters(args), ctx).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::Custom(-32002));
}
```

**Success Criteria**: User Story 2 and User Story 5 acceptance scenarios pass.

---

## Phase 1c: Resources Foundation

**Goal**: Implement resource support with market data resources

### Step 1: Add Resource URI Parser

**File**: `src/server/resources.rs` (create new file)

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceCategory {
    Market,
    Account,
    Orders,
}

pub struct ResourceUri {
    pub scheme: String,
    pub category: ResourceCategory,
    pub identifier: Option<String>,
}

impl ResourceUri {
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

---

### Step 2: Implement Resource Handlers

**File**: `src/server/handler.rs`

Add resource methods to `ServerHandler` impl:

```rust
use rmcp::model::{Resource, ResourceContents};
use crate::server::resources::{ResourceUri, ResourceCategory};

impl ServerHandler for BinanceServer {
    // ... existing get_info, prompts

    async fn list_resources(&self) -> Result<Vec<Resource>, ErrorData> {
        Ok(vec![
            Resource {
                uri: "binance://market/btcusdt".to_string(),
                name: "BTCUSDT Market Data".to_string(),
                description: Some("Real-time 24-hour ticker statistics for Bitcoin/USDT trading pair".to_string()),
                mime_type: Some("text/markdown".to_string()),
            },
            Resource {
                uri: "binance://market/ethusdt".to_string(),
                name: "ETHUSDT Market Data".to_string(),
                description: Some("Real-time 24-hour ticker statistics for Ethereum/USDT trading pair".to_string()),
                mime_type: Some("text/markdown".to_string()),
            },
        ])
    }

    async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContents>, ErrorData> {
        let parsed = ResourceUri::parse(uri)
            .map_err(|e| ErrorData::new(-32404, format!("Invalid resource URI: {}", e)))?;

        match parsed.category {
            ResourceCategory::Market => self.read_market_resource(parsed.identifier).await,
            ResourceCategory::Account => self.read_account_resource(parsed.identifier).await,
            ResourceCategory::Orders => self.read_orders_resource(parsed.identifier).await,
        }
    }
}

impl BinanceServer {
    async fn read_market_resource(&self, identifier: Option<String>) -> Result<Vec<ResourceContents>, ErrorData> {
        let symbol = identifier.ok_or_else(|| {
            ErrorData::new(-32404, "Market resource requires symbol identifier".to_string())
        })?;

        // Normalize to uppercase for API
        let symbol_upper = symbol.to_uppercase();

        let ticker = self.binance_client
            .get_24hr_ticker(&symbol_upper)
            .await
            .map_err(|e| BinanceError::ApiError(e).into())?;

        let content = format!(
            "# {} Market Data\n\n\
            **Symbol**: {}\n\
            **Last Price**: ${}\n\
            **24h Change**: {} ({}%)\n\
            **24h High**: ${}\n\
            **24h Low**: ${}\n\
            **24h Volume**: {} {}\n\
            **Quote Volume**: ${} USDT\n\n\
            *Last updated: {}*\n\
            *Data source: Binance API v3*",
            ticker.symbol,
            ticker.symbol,
            ticker.last_price,
            ticker.price_change,
            ticker.price_change_percent,
            ticker.high_price,
            ticker.low_price,
            ticker.volume,
            ticker.symbol.trim_end_matches("USDT"),
            ticker.quote_volume,
            chrono::Utc::now().to_rfc3339(),
        );

        Ok(vec![ResourceContents {
            uri: format!("binance://market/{}", symbol),
            mime_type: Some("text/markdown".to_string()),
            text: Some(content),
            blob: None,
        }])
    }

    async fn read_account_resource(&self, _identifier: Option<String>) -> Result<Vec<ResourceContents>, ErrorData> {
        // TODO: Phase 1d implementation
        Err(ErrorData::new(-32404, "Not implemented".to_string()))
    }

    async fn read_orders_resource(&self, _identifier: Option<String>) -> Result<Vec<ResourceContents>, ErrorData> {
        // TODO: Phase 1d implementation
        Err(ErrorData::new(-32404, "Not implemented".to_string()))
    }
}
```

**Update capabilities**:

```rust
fn get_info(&self) -> InitializeResult {
    // ... existing code
    capabilities: ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(false) }),
        prompts: Some(PromptsCapability { list_changed: Some(false) }),
        resources: Some(ResourcesCapability { subscribe: Some(false), list_changed: Some(false) }),
        ..Default::default()
    },
    // ...
}
```

---

### Step 3: Test Phase 1c

**Manual Test**:

1. In Claude Desktop, type: `binance://market/btcusdt`

2. Expected: Claude accesses resource and displays formatted market data.

**Integration Test**:

```rust
#[tokio::test]
async fn test_list_resources() {
    let server = BinanceServer::new(/* testnet */);
    let resources = server.list_resources().await.unwrap();
    assert!(resources.len() >= 2);
}

#[tokio::test]
async fn test_read_market_resource() {
    let server = BinanceServer::new(/* testnet */);
    let contents = server.read_resource("binance://market/btcusdt").await.unwrap();

    assert_eq!(contents.len(), 1);
    assert!(contents[0].text.as_ref().unwrap().contains("BTCUSDT"));
}
```

**Success Criteria**: User Story 3 acceptance scenarios pass.

---

## Phase 1d: Account Resources

**Goal**: Add account balance and open orders resources

### Step 1: Extend Resource List

**File**: `src/server/handler.rs`

Update `list_resources()`:

```rust
async fn list_resources(&self) -> Result<Vec<Resource>, ErrorData> {
    Ok(vec![
        // ... existing market resources
        Resource {
            uri: "binance://account/balances".to_string(),
            name: "Account Balances".to_string(),
            description: Some("Current account balances for all assets (free and locked)".to_string()),
            mime_type: Some("text/markdown".to_string()),
        },
        Resource {
            uri: "binance://orders/open".to_string(),
            name: "Open Orders".to_string(),
            description: Some("All currently active orders (NEW, PARTIALLY_FILLED)".to_string()),
            mime_type: Some("text/markdown".to_string()),
        },
    ])
}
```

---

### Step 2: Implement Account/Orders Handlers

```rust
async fn read_account_resource(&self, identifier: Option<String>) -> Result<Vec<ResourceContents>, ErrorData> {
    match identifier.as_deref() {
        Some("balances") | None => {
            let account = self.binance_client.get_account_info().await
                .map_err(|e| BinanceError::ApiError(e).into())?;

            let balances: Vec<_> = account.balances.iter()
                .filter(|b| b.free.parse::<f64>().unwrap_or(0.0) > 0.0 || b.locked.parse::<f64>().unwrap_or(0.0) > 0.0)
                .collect();

            let mut content = String::from("# Account Balances\n\n");
            content.push_str("| Asset | Free | Locked | Total |\n");
            content.push_str("|-------|------|--------|-------|\n");

            for balance in &balances {
                let total = balance.free.parse::<f64>().unwrap_or(0.0) + balance.locked.parse::<f64>().unwrap_or(0.0);
                content.push_str(&format!("| {} | {} | {} | {:.8} |\n",
                    balance.asset, balance.free, balance.locked, total));
            }

            content.push_str(&format!("\n*Last updated: {}*", chrono::Utc::now().to_rfc3339()));

            Ok(vec![ResourceContents {
                uri: "binance://account/balances".to_string(),
                mime_type: Some("text/markdown".to_string()),
                text: Some(content),
                blob: None,
            }])
        },
        _ => Err(ErrorData::new(-32404, "Unknown account resource".to_string())),
    }
}

async fn read_orders_resource(&self, identifier: Option<String>) -> Result<Vec<ResourceContents>, ErrorData> {
    match identifier.as_deref() {
        Some("open") | None => {
            let orders = self.binance_client.get_open_orders(None).await
                .map_err(|e| BinanceError::ApiError(e).into())?;

            let mut content = String::from("# Open Orders\n\n");

            if orders.is_empty() {
                content.push_str("No open orders found.\n");
            } else {
                content.push_str("| Order ID | Symbol | Side | Type | Price | Qty | Status |\n");
                content.push_str("|----------|--------|------|------|-------|-----|--------|\n");

                for order in &orders {
                    content.push_str(&format!("| {} | {} | {} | {} | {} | {} | {} |\n",
                        order.order_id, order.symbol, order.side, order.order_type,
                        order.price, order.orig_qty, order.status));
                }
            }

            content.push_str(&format!("\n*Last updated: {}*", chrono::Utc::now().to_rfc3339()));

            Ok(vec![ResourceContents {
                uri: "binance://orders/open".to_string(),
                mime_type: Some("text/markdown".to_string()),
                text: Some(content),
                blob: None,
            }])
        },
        _ => Err(ErrorData::new(-32404, "Unknown orders resource".to_string())),
    }
}
```

---

### Step 3: Test Phase 1d

**Manual Test**:

1. Read `binance://account/balances` → Verify balance table
2. Read `binance://orders/open` → Verify orders list (or "No open orders")

**Integration Test**:

```rust
#[tokio::test]
async fn test_read_account_balances() {
    let server = BinanceServer::new(/* testnet */);
    let contents = server.read_resource("binance://account/balances").await.unwrap();
    assert!(contents[0].text.as_ref().unwrap().contains("Account Balances"));
}

#[tokio::test]
async fn test_read_open_orders() {
    let server = BinanceServer::new(/* testnet */);
    let contents = server.read_resource("binance://orders/open").await.unwrap();
    assert!(contents[0].text.as_ref().unwrap().contains("Open Orders"));
}
```

**Success Criteria**: User Story 4 acceptance scenarios pass.

---

## Final Verification

### Run All Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Clippy (linting)
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

### Manual Verification (Claude Desktop)

1. **Prompts**:
   - "Analyze BTCUSDT for a balanced trading strategy"
   - "What's my portfolio risk?"

2. **Resources**:
   - Access `binance://market/btcusdt`
   - Access `binance://account/balances`
   - Access `binance://orders/open`

3. **Errors**:
   - Trigger rate limit (call get_ticker 100 times)
   - Use invalid symbol (BTC-USD)
   - Use invalid credentials

### Success Criteria Checklist

- [ ] SC-001: Trading analysis within 3 seconds
- [ ] SC-002: Portfolio risk assessment complete
- [ ] SC-003: 40% reduction in tool calls (resources vs tools)
- [ ] SC-004: 90% of errors have recovery suggestions
- [ ] SC-005: At least 5 resources listed
- [ ] SC-007: Complete market context in prompts
- [ ] SC-008: Concurrent prompt/resource/tool requests

---

## Troubleshooting

### Error: "prompt_handler macro not found"

**Solution**: Ensure rmcp version is 0.8.1+:
```bash
cargo update rmcp
```

### Error: "Multiple tool_handler/prompt_handler attributes"

**Solution**: Combine macros on same impl block:
```rust
#[tool_handler(router = self.tool_router)]
#[prompt_handler]
impl ServerHandler for BinanceServer { ... }
```

### Prompts not appearing in Claude Desktop

**Solution**: Check capabilities in `get_info()`:
```rust
prompts: Some(PromptsCapability { list_changed: Some(false) }),
```

### Resources not listed

**Solution**: Verify capabilities:
```rust
resources: Some(ResourcesCapability { subscribe: Some(false), list_changed: Some(false) }),
```

---

## Next Steps

After completing all phases:

1. **Generate Tasks**: Run `/speckit.tasks` to create implementation tasks
2. **Implement**: Execute tasks via `/speckit.implement`
3. **Update Documentation**: Update README.md with new features
4. **Create PR**: Merge `006-prompts-resources-errors` into main

---

**Quickstart Status**: ✅ Complete - Ready for Implementation
