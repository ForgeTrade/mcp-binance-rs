//! MCP ServerHandler trait implementation
//!
//! Implements the MCP protocol ServerHandler trait for the Binance server.
//! Provides server info, capabilities, and lifecycle management.

use crate::server::BinanceServer;
use crate::server::resources::{ResourceCategory, ResourceUri};
use crate::server::types::{PortfolioRiskArgs, TradingAnalysisArgs};
use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    AnnotateAble, ErrorData, GetPromptRequestParam, GetPromptResult, Implementation,
    InitializeResult, ListPromptsResult, ListResourcesResult, PaginatedRequestParam, PromptMessage,
    PromptMessageRole, PromptsCapability, ProtocolVersion, RawResource, ReadResourceRequestParam,
    ReadResourceResult, ResourceContents, ResourcesCapability, ServerCapabilities, ToolsCapability,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, prompt, prompt_handler, prompt_router, tool_handler};

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for BinanceServer {
    /// Returns server information and capabilities
    ///
    /// This is called during MCP initialization to communicate server metadata
    /// and supported features to the client.
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
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
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

    /// List available resources (T028)
    ///
    /// Returns a list of all available MCP resources for market data.
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: vec![
                // Market data resources
                RawResource {
                    uri: "binance://market/btcusdt".to_string(),
                    name: "BTCUSDT Market Data".to_string(),
                    title: None,
                    description: Some(
                        "Real-time 24-hour ticker statistics for Bitcoin/USDT trading pair"
                            .to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                    icons: None,
                }
                .no_annotation(),
                RawResource {
                    uri: "binance://market/ethusdt".to_string(),
                    name: "ETHUSDT Market Data".to_string(),
                    title: None,
                    description: Some(
                        "Real-time 24-hour ticker statistics for Ethereum/USDT trading pair"
                            .to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                    icons: None,
                }
                .no_annotation(),
                // Account resources (T035)
                RawResource {
                    uri: "binance://account/balances".to_string(),
                    name: "Account Balances".to_string(),
                    title: None,
                    description: Some(
                        "Current account balances with free and locked amounts for all assets"
                            .to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                    icons: None,
                }
                .no_annotation(),
                // Orders resources (T035)
                RawResource {
                    uri: "binance://orders/open".to_string(),
                    name: "Open Orders".to_string(),
                    title: None,
                    description: Some(
                        "List of all currently open orders across all trading pairs".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                    icons: None,
                }
                .no_annotation(),
            ],
            next_cursor: None,
        })
    }

    /// Read a specific resource by URI (T029)
    ///
    /// Parses the URI and dispatches to the appropriate resource handler.
    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        // Parse URI (T032 - error handling)
        let parsed = ResourceUri::parse(&request.uri).map_err(|e| {
            ErrorData::new(
                rmcp::model::ErrorCode(-32404),
                format!("Invalid resource URI: {}", e),
                Some(serde_json::json!({
                    "provided_uri": request.uri,
                    "valid_examples": [
                        "binance://market/btcusdt",
                        "binance://market/ethusdt",
                        "binance://account/balances",
                        "binance://orders/open"
                    ],
                    "recovery_suggestion": "Use format: binance://{category}/{identifier}"
                })),
            )
        })?;

        // Dispatch to category-specific handlers
        let contents = match parsed.category {
            ResourceCategory::Market => self.read_market_resource(parsed.identifier).await?,
            ResourceCategory::Account => self.read_account_resource(parsed.identifier).await?, // T036
            ResourceCategory::Orders => self.read_orders_resource(parsed.identifier).await?, // T037
        };

        Ok(ReadResourceResult { contents })
    }
}

/// Resource handler implementation
impl BinanceServer {
    /// Read market data resource (T030, T031, T034)
    ///
    /// Fetches 24hr ticker data for the specified symbol and formats it as markdown.
    async fn read_market_resource(
        &self,
        identifier: Option<String>,
    ) -> Result<Vec<ResourceContents>, ErrorData> {
        // Require symbol identifier
        let symbol = identifier.ok_or_else(|| {
            ErrorData::new(
                rmcp::model::ErrorCode(-32404),
                "Market resource requires symbol identifier".to_string(),
                Some(serde_json::json!({
                    "valid_examples": ["binance://market/btcusdt", "binance://market/ethusdt"],
                    "recovery_suggestion": "Specify symbol: binance://market/{symbol}"
                })),
            )
        })?;

        // Normalize to uppercase for API (T031)
        let symbol_upper = symbol.to_uppercase();

        // Fetch ticker data
        let ticker = self
            .binance_client
            .get_24hr_ticker(&symbol_upper)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to fetch market data: {}", e), None)
            })?;

        // Format as markdown
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
            chrono::Utc::now().to_rfc3339() // T034 timestamp
        );

        Ok(vec![ResourceContents::TextResourceContents {
            uri: format!("binance://market/{}", symbol),
            mime_type: Some("text/markdown".to_string()),
            text: content,
            meta: None,
        }])
    }

    /// Read account resource (T036, T038)
    ///
    /// Fetches account information and formats balances as markdown table.
    async fn read_account_resource(
        &self,
        identifier: Option<String>,
    ) -> Result<Vec<ResourceContents>, ErrorData> {
        // Require "balances" identifier
        let id = identifier.ok_or_else(|| {
            ErrorData::new(
                rmcp::model::ErrorCode(-32404),
                "Account resource requires identifier".to_string(),
                Some(serde_json::json!({
                    "valid_examples": ["binance://account/balances"],
                    "recovery_suggestion": "Specify identifier: binance://account/{identifier}"
                })),
            )
        })?;

        if id != "balances" {
            return Err(ErrorData::new(
                rmcp::model::ErrorCode(-32404),
                format!("Unknown account identifier: '{}'", id),
                Some(serde_json::json!({
                    "provided_identifier": id,
                    "valid_identifiers": ["balances"],
                    "recovery_suggestion": "Use 'balances' for account balance information"
                })),
            ));
        }

        // Fetch account information
        let account = self.binance_client.get_account().await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to fetch account info: {}", e), None)
        })?;

        // Filter non-zero balances (T038)
        let balances: Vec<_> = account
            .balances
            .iter()
            .filter(|b| {
                let free = b.free.parse::<f64>().unwrap_or(0.0);
                let locked = b.locked.parse::<f64>().unwrap_or(0.0);
                free > 0.0 || locked > 0.0
            })
            .collect();

        // Format as markdown table (T038)
        let mut content = String::from("# Account Balances\n\n");

        if balances.is_empty() {
            content.push_str("No active balances found in your account.\n\n");
            content.push_str("**Note**: Only assets with non-zero balances are displayed.\n");
        } else {
            content.push_str("| Asset | Free Balance | Locked Balance | Total |\n");
            content.push_str("|-------|--------------|----------------|-------|\n");

            for balance in &balances {
                let free = balance.free.parse::<f64>().unwrap_or(0.0);
                let locked = balance.locked.parse::<f64>().unwrap_or(0.0);
                let total = free + locked;

                content.push_str(&format!(
                    "| {} | {} | {} | {:.8} |\n",
                    balance.asset, balance.free, balance.locked, total
                ));
            }

            content.push_str(&format!("\n**Total Assets**: {}\n", balances.len()));
        }

        // Add timestamp
        content.push_str(&format!(
            "\n*Last updated: {}*\n\
            *Data source: Binance API v3*\n\
            *Note: Testnet account - no real funds*",
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(vec![ResourceContents::TextResourceContents {
            uri: "binance://account/balances".to_string(),
            mime_type: Some("text/markdown".to_string()),
            text: content,
            meta: None,
        }])
    }

    /// Read orders resource (T037, T039, T040)
    ///
    /// Fetches open orders and formats as markdown table.
    async fn read_orders_resource(
        &self,
        identifier: Option<String>,
    ) -> Result<Vec<ResourceContents>, ErrorData> {
        // Require "open" identifier
        let id = identifier.ok_or_else(|| {
            ErrorData::new(
                rmcp::model::ErrorCode(-32404),
                "Orders resource requires identifier".to_string(),
                Some(serde_json::json!({
                    "valid_examples": ["binance://orders/open"],
                    "recovery_suggestion": "Specify identifier: binance://orders/{identifier}"
                })),
            )
        })?;

        if id != "open" {
            return Err(ErrorData::new(
                rmcp::model::ErrorCode(-32404),
                format!("Unknown orders identifier: '{}'", id),
                Some(serde_json::json!({
                    "provided_identifier": id,
                    "valid_identifiers": ["open"],
                    "recovery_suggestion": "Use 'open' for currently open orders"
                })),
            ));
        }

        // Fetch open orders (all symbols)
        let orders = self
            .binance_client
            .get_open_orders(None)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to fetch open orders: {}", e), None)
            })?;

        // Format as markdown table (T039, T040)
        let mut content = String::from("# Open Orders\n\n");

        if orders.is_empty() {
            // Handle empty orders case (T040)
            content.push_str("No open orders found.\n\n");
            content.push_str("**Note**: This includes all trading pairs on your account.\n");
        } else {
            content.push_str("| Order ID | Symbol | Side | Type | Price | Quantity | Status |\n");
            content.push_str("|----------|--------|------|------|-------|----------|--------|\n");

            for order in &orders {
                content.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} |\n",
                    order.order_id,
                    order.symbol,
                    order.side,
                    order.order_type,
                    order.price,
                    order.orig_qty,
                    order.status
                ));
            }

            content.push_str(&format!("\n**Total Open Orders**: {}\n", orders.len()));
        }

        // Add timestamp
        content.push_str(&format!(
            "\n*Last updated: {}*\n\
            *Data source: Binance API v3*\n\
            *Note: Testnet account*",
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(vec![ResourceContents::TextResourceContents {
            uri: "binance://orders/open".to_string(),
            mime_type: Some("text/markdown".to_string()),
            text: content,
            meta: None,
        }])
    }
}

/// Prompt definitions for AI-guided trading analysis and portfolio assessment
#[prompt_router]
impl BinanceServer {
    /// Creates a prompt router with all registered prompts
    pub fn create_prompt_router() -> PromptRouter<Self> {
        Self::prompt_router()
    }

    /// AI-guided trading analysis prompt
    ///
    /// Analyzes market conditions for a specific cryptocurrency and provides
    /// trading recommendations based on 24-hour ticker data.
    #[prompt(
        name = "trading_analysis",
        description = "Analyze market conditions for a specific cryptocurrency and provide trading recommendations"
    )]
    pub async fn trading_analysis(
        &self,
        Parameters(args): Parameters<TradingAnalysisArgs>,
    ) -> Result<GetPromptResult, ErrorData> {
        // Fetch 24hr ticker data
        let ticker = self
            .binance_client
            .get_24hr_ticker(&args.symbol)
            .await
            .map_err(|e| {
                // Convert McpError to ErrorData
                ErrorData::internal_error(format!("Failed to fetch ticker data: {}", e), None)
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

        // Add timestamp
        content.push_str(&format!(
            "\n*Last updated: {}*\n\
            *Data source: Binance API v3*",
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(GetPromptResult {
            description: None,
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
        })
    }

    /// Portfolio risk assessment prompt
    ///
    /// Analyzes current holdings and provides portfolio diversification recommendations
    /// based on account balances and risk exposure.
    #[prompt(
        name = "portfolio_risk",
        description = "Assess portfolio risk and provide diversification recommendations based on current holdings"
    )]
    pub async fn portfolio_risk(
        &self,
        Parameters(_args): Parameters<PortfolioRiskArgs>,
    ) -> Result<GetPromptResult, ErrorData> {
        // Fetch account information
        let account = self.binance_client.get_account().await.map_err(|e| {
            // Convert McpError to ErrorData
            ErrorData::internal_error(format!("Failed to fetch account info: {}", e), None)
        })?;

        // Filter non-zero balances (T022)
        let balances: Vec<_> = account
            .balances
            .iter()
            .filter(|b| {
                let free = b.free.parse::<f64>().unwrap_or(0.0);
                let locked = b.locked.parse::<f64>().unwrap_or(0.0);
                free > 0.0 || locked > 0.0
            })
            .collect();

        let mut content = String::from("# Portfolio Risk Assessment\n\n## Current Holdings\n\n");

        // Handle empty portfolio case (T023)
        if balances.is_empty() {
            content.push_str("No active balances found in your account.\n\n");
            content.push_str("**Recommendation**: Deposit funds to begin trading. Start with:\n");
            content.push_str("- 30% stablecoins (USDT/BUSD) for liquidity\n");
            content.push_str("- 40% major cryptocurrencies (BTC/ETH)\n");
            content.push_str("- 30% diversified altcoins based on risk tolerance\n");
        } else {
            // Format balance table (T022)
            content.push_str("| Asset | Free Balance | Locked Balance | Total |\n");
            content.push_str("|-------|--------------|----------------|-------|\n");

            for balance in &balances {
                let free = balance.free.parse::<f64>().unwrap_or(0.0);
                let locked = balance.locked.parse::<f64>().unwrap_or(0.0);
                let total = free + locked;

                content.push_str(&format!(
                    "| {} | {} | {} | {:.8} |\n",
                    balance.asset, balance.free, balance.locked, total
                ));
            }

            content.push_str(&format!("\n**Total Assets**: {}\n", balances.len()));
        }

        // Add timestamp (T024)
        content.push_str(&format!(
            "\n*Last updated: {}*\n\
            *Note: Testnet account - no real funds*",
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(GetPromptResult {
            description: None,
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
        })
    }
}
