# MCP Binance Server - Improvement Recommendations

Based on analysis of MCP documentation, rmcp examples, and official Binance connector.

## 🎯 High Priority Improvements

### 1. Add MCP Prompts Support

**Current State:** Only tools are implemented
**Improvement:** Add `#[prompt_router]` for AI-guided trading workflows

**Example Implementation:**
```rust
#[prompt_router]
impl BinanceServer {
    /// Analyze market conditions and suggest trading strategy
    #[prompt(name = "trading_analysis")]
    async fn trading_analysis(
        &self,
        Parameters(args): Parameters<TradingAnalysisArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let ticker = self.binance_client.get_24hr_ticker(&args.symbol).await?;

        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Analyze trading conditions for {}:\n\
                     Current price: {}\n\
                     24h change: {}%\n\
                     Volume: {}\n\
                     High/Low: {} / {}\n\n\
                     Strategy preference: {}\n\
                     Risk tolerance: {}\n\n\
                     Provide trading recommendation.",
                    args.symbol,
                    ticker.last_price,
                    ticker.price_change_percent,
                    ticker.volume,
                    ticker.high_price,
                    ticker.low_price,
                    args.strategy.unwrap_or("balanced".to_string()),
                    args.risk_level.unwrap_or("medium".to_string())
                )
            )
        ];

        Ok(GetPromptResult {
            description: Some(format!("Trading analysis for {}", args.symbol)),
            messages,
        })
    }

    /// Portfolio risk assessment
    #[prompt(name = "portfolio_risk")]
    async fn portfolio_risk(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let account = self.binance_client.get_account().await?;

        // Format balances for analysis
        let balances: Vec<String> = account.balances.iter()
            .filter(|b| b.free.parse::<f64>().unwrap_or(0.0) > 0.0)
            .map(|b| format!("{}: {} (free), {} (locked)",
                b.asset, b.free, b.locked))
            .collect();

        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Analyze portfolio risk:\n\n\
                     Current Holdings:\n{}\n\n\
                     Provide risk assessment and diversification recommendations.",
                    balances.join("\n")
                )
            )
        ];

        Ok(GetPromptResult {
            description: Some("Portfolio risk analysis".to_string()),
            messages,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TradingAnalysisArgs {
    /// Trading pair to analyze
    pub symbol: String,
    /// Strategy preference: aggressive, balanced, conservative
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    /// Risk tolerance: low, medium, high
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<String>,
}
```

**Benefits:**
- Claude can provide contextual trading advice
- Natural language interface for complex analysis
- Combines real market data with AI reasoning

---

### 2. Add MCP Resources Support

**Current State:** No resources exposed
**Improvement:** Expose market data and account info as resources

**Example Implementation:**
```rust
#[tool_handler]
#[prompt_handler]
impl ServerHandler for BinanceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()  // ✅ Add this
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("...".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: vec![
                // Market data resources
                RawResource::new(
                    "binance://market/btcusdt",
                    "Bitcoin/USDT Market Data"
                ).no_annotation(),
                RawResource::new(
                    "binance://market/ethusdt",
                    "Ethereum/USDT Market Data"
                ).no_annotation(),

                // Account resources
                RawResource::new(
                    "binance://account/balances",
                    "Account Balances"
                ).no_annotation(),
                RawResource::new(
                    "binance://account/positions",
                    "Open Positions"
                ).no_annotation(),

                // Order resources
                RawResource::new(
                    "binance://orders/open",
                    "Open Orders"
                ).no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        match uri.as_str() {
            uri if uri.starts_with("binance://market/") => {
                let symbol = uri.strip_prefix("binance://market/")
                    .unwrap_or("")
                    .to_uppercase();

                let ticker = self.binance_client
                    .get_24hr_ticker(&symbol)
                    .await
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

                let content = format!(
                    "# {} Market Data\n\n\
                     **Price:** {} USDT\n\
                     **24h Change:** {}%\n\
                     **24h Volume:** {} {}\n\
                     **High/Low:** {} / {}\n\
                     **Last Update:** {}",
                    symbol,
                    ticker.last_price,
                    ticker.price_change_percent,
                    ticker.volume,
                    symbol.strip_suffix("USDT").unwrap_or(&symbol),
                    ticker.high_price,
                    ticker.low_price,
                    chrono::Utc::now().to_rfc3339()
                );

                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(&content, uri)],
                })
            }

            "binance://account/balances" => {
                let account = self.binance_client
                    .get_account()
                    .await
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

                let content = format!(
                    "# Account Balances\n\n\
                     {}\n\n\
                     **Can Trade:** {}\n\
                     **Last Update:** {}",
                    account.balances.iter()
                        .filter(|b| b.free.parse::<f64>().unwrap_or(0.0) > 0.0)
                        .map(|b| format!("- **{}:** {} (free), {} (locked)",
                            b.asset, b.free, b.locked))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    account.can_trade,
                    chrono::Utc::now().to_rfc3339()
                );

                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(&content, uri)],
                })
            }

            _ => Err(ErrorData::resource_not_found(
                "Resource not found",
                Some(json!({"uri": uri}))
            ))
        }
    }
}
```

**Benefits:**
- Claude can access market data directly without tool calls
- Resources are cached and efficiently retrieved
- Better for displaying contextual information

---

### 3. Improve Error Handling with Context

**Current State:** Basic error conversion
**Improvement:** Rich error context with recovery suggestions

**Implementation:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinanceError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Rate limit exceeded. Retry after {retry_after}s. Current weight: {weight}")]
    RateLimited {
        weight: u32,
        retry_after: u64,
        limit: u32,
    },

    #[error("Invalid API credentials. Check BINANCE_API_KEY and BINANCE_SECRET_KEY")]
    InvalidCredentials,

    #[error("API error [{code}]: {message}")]
    ApiError { code: i32, message: String },

    #[error("Symbol {symbol} not found or invalid")]
    InvalidSymbol { symbol: String },

    #[error("Insufficient balance. Required: {required}, Available: {available}")]
    InsufficientBalance {
        asset: String,
        required: String,
        available: String,
    },
}

impl From<BinanceError> for ErrorData {
    fn from(err: BinanceError) -> Self {
        use BinanceError::*;

        match err {
            RateLimited { weight, retry_after, limit } => {
                ErrorData::rate_limited(
                    &format!("Rate limit exceeded. Retry after {}s", retry_after),
                    Some(json!({
                        "weight": weight,
                        "limit": limit,
                        "retry_after": retry_after,
                        "suggestion": "Wait before making more requests or reduce request frequency"
                    }))
                )
            }

            InvalidCredentials => {
                ErrorData::invalid_params(
                    "Invalid API credentials",
                    Some(json!({
                        "suggestion": "Check environment variables BINANCE_API_KEY and BINANCE_SECRET_KEY",
                        "docs": "https://testnet.binance.vision/"
                    }))
                )
            }

            InvalidSymbol { symbol } => {
                ErrorData::invalid_params(
                    &format!("Invalid symbol: {}", symbol),
                    Some(json!({
                        "symbol": symbol,
                        "suggestion": "Use format like BTCUSDT, ETHUSDT. Check /api/v3/exchangeInfo for valid symbols"
                    }))
                )
            }

            _ => ErrorData::internal_error(err.to_string(), None)
        }
    }
}
```

**Benefits:**
- Better user experience with actionable error messages
- Debug information included for troubleshooting
- Recovery suggestions built-in

---

### 4. Add Progress Reporting for Long Operations

**Current State:** No progress feedback
**Improvement:** Report progress for multi-step operations

**Example:**
```rust
#[tool(description = "Batch place multiple orders with progress updates")]
pub async fn batch_place_orders(
    &self,
    params: Parameters<BatchOrdersParam>,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, ErrorData> {
    let total = params.0.orders.len();
    let mut results = Vec::new();

    for (index, order) in params.0.orders.iter().enumerate() {
        // Send progress notification
        ctx.send_progress(
            format!("Placing order {} of {}", index + 1, total),
            Some((index as f64 + 1.0) / total as f64)
        ).await;

        let result = self.binance_client.create_order(
            &order.symbol,
            &order.side,
            &order.order_type,
            &order.quantity,
            order.price.as_deref(),
        ).await;

        results.push(result);

        // Small delay to avoid rate limits
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Return summary
    let success_count = results.iter().filter(|r| r.is_ok()).count();

    Ok(CallToolResult::success(vec![Content::text(
        format!("Batch complete: {}/{} orders placed successfully",
            success_count, total)
    )]))
}
```

---

## 🚀 Medium Priority Improvements

### 5. WebSocket Support for Real-Time Data

**Rationale:** Binance provides WebSocket API for real-time updates

**Implementation Strategy:**
1. Add `websocket` feature flag to Cargo.toml
2. Create WebSocket client in `src/binance/websocket.rs`
3. Add streaming tools for price updates

**Example:**
```rust
#[tool(description = "Subscribe to real-time price updates")]
pub async fn subscribe_price_stream(
    &self,
    params: Parameters<StreamParam>,
) -> Result<CallToolResult, ErrorData> {
    // Start WebSocket stream in background
    let (tx, mut rx) = mpsc::channel(100);

    let ws_url = format!(
        "wss://stream.binance.com:9443/ws/{}@ticker",
        params.0.symbol.to_lowercase()
    );

    // Spawn WebSocket listener
    tokio::spawn(async move {
        // WebSocket connection and message handling
    });

    Ok(CallToolResult::success(vec![Content::text(
        format!("Subscribed to {} price stream", params.0.symbol)
    )]))
}
```

---

### 6. Caching Layer for Frequently Accessed Data

**Rationale:** Reduce API calls and improve response time

**Implementation:**
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CachedData<T> {
    data: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CachedData<T> {
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

pub struct BinanceClient {
    // ... existing fields
    ticker_cache: Arc<RwLock<HashMap<String, CachedData<Ticker24hr>>>>,
}

impl BinanceClient {
    pub async fn get_24hr_ticker_cached(&self, symbol: &str) -> Result<Ticker24hr> {
        // Check cache first
        {
            let cache = self.ticker_cache.read().await;
            if let Some(cached) = cache.get(symbol) {
                if cached.is_valid() {
                    tracing::debug!("Cache hit for {}", symbol);
                    return Ok(cached.data.clone());
                }
            }
        }

        // Cache miss - fetch from API
        let ticker = self.get_24hr_ticker(symbol).await?;

        // Update cache
        {
            let mut cache = self.ticker_cache.write().await;
            cache.insert(symbol.to_string(), CachedData {
                data: ticker.clone(),
                cached_at: Instant::now(),
                ttl: Duration::from_secs(5), // 5 second TTL
            });
        }

        Ok(ticker)
    }
}
```

---

### 7. Enhanced Logging with Structured Fields

**Current State:** Basic tracing
**Improvement:** Rich structured logging

**Implementation:**
```rust
use tracing::{info, warn, error, instrument};

impl BinanceClient {
    #[instrument(
        skip(self),
        fields(
            symbol = %symbol,
            endpoint = "/api/v3/ticker/24hr"
        )
    )]
    pub async fn get_24hr_ticker(&self, symbol: &str) -> Result<Ticker24hr> {
        let start = Instant::now();

        let response = self.client
            .get(format!("{}/api/v3/ticker/24hr", self.base_url))
            .query(&[("symbol", symbol)])
            .send()
            .await?;

        let elapsed = start.elapsed();

        // Log response metadata
        let status = response.status();
        let headers = response.headers();

        if let Some(weight) = headers.get("x-mbx-used-weight-1m") {
            tracing::info!(
                weight = %weight.to_str().unwrap_or("unknown"),
                "Rate limit weight used"
            );
        }

        tracing::info!(
            status = %status,
            latency_ms = elapsed.as_millis(),
            "API request completed"
        );

        response.json().await
    }
}
```

---

## 🎨 Low Priority / Nice to Have

### 8. Configuration Hot-Reload

**Benefit:** Change config without restarting server

### 9. Metrics Collection

**Benefit:** Monitor performance and usage patterns

### 10. Multiple Exchange Support

**Benefit:** Abstract to support other exchanges (Coinbase, Kraken)

---

## 📊 Architecture Improvements

### Current Architecture:
```
src/
├── main.rs           # Entry point
├── server/
│   ├── mod.rs        # BinanceServer struct
│   ├── handler.rs    # ServerHandler impl
│   └── tool_router.rs # All 13 tools
├── binance/
│   ├── client.rs     # HTTP client
│   └── types.rs      # Response types
├── config/
│   └── mod.rs        # Config loading
└── error.rs          # Error types
```

### Recommended Architecture:
```
src/
├── main.rs
├── server/
│   ├── mod.rs
│   ├── handler.rs
│   ├── tool_router.rs    # Tools
│   ├── prompt_router.rs  # ✅ NEW: Prompts
│   └── resources.rs      # ✅ NEW: Resources
├── binance/
│   ├── client.rs
│   ├── websocket.rs      # ✅ NEW: WebSocket
│   ├── cache.rs          # ✅ NEW: Caching
│   └── types.rs
├── config/
│   ├── mod.rs
│   └── hot_reload.rs     # ✅ NEW: Hot reload
├── middleware/
│   ├── rate_limit.rs     # ✅ NEW: Rate limiting
│   └── retry.rs          # ✅ NEW: Retry logic
├── metrics/              # ✅ NEW: Metrics
│   └── collector.rs
└── error.rs
```

---

## 🎯 Implementation Priority

**Phase 1 (Immediate):**
1. ✅ Add Prompts support
2. ✅ Add Resources support
3. ✅ Improve error handling

**Phase 2 (Short-term):**
4. Add progress reporting
5. Add caching layer
6. Enhanced logging

**Phase 3 (Medium-term):**
7. WebSocket support
8. Rate limit middleware
9. Metrics collection

**Phase 4 (Long-term):**
10. Configuration hot-reload
11. Multi-exchange support
12. Advanced features

---

## 📚 References

- MCP Specification: https://spec.modelcontextprotocol.io/
- rmcp Examples: `/rust-sdk/examples/servers/`
- Counter Example: `/rust-sdk/examples/servers/src/common/counter.rs`
- Progress Demo: `/rust-sdk/examples/servers/src/progress_demo.rs`
- Binance SDK: `/binance-connector-rust/`
- Building MCP with LLMs: `/modelcontextprotocol/docs/tutorials/building-mcp-with-llms.mdx`

---

**Generated:** 2025-10-17
**Version:** 1.0.0
**Status:** Ready for implementation
