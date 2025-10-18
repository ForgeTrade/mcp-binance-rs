//! MCP ServerHandler trait implementation
//!
//! Implements the MCP protocol ServerHandler trait for the Binance server.
//! Provides server info, capabilities, and lifecycle management.

#[cfg(feature = "orderbook_analytics")]
use crate::orderbook::analytics::types::FlowDirection;
use crate::server::BinanceServer;
use crate::server::resources::{ResourceCategory, ResourceUri};
#[cfg(feature = "orderbook_analytics")]
use crate::server::types::{AdvancedAnalysisArgs, MarketHealthCheckArgs, OrderFlowSnapshotArgs};
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

    /// Advanced market analysis prompt using orderbook analytics
    ///
    /// Provides comprehensive market analysis combining order flow, volume profile,
    /// anomaly detection, and health scoring for professional trading decisions.
    #[cfg(feature = "orderbook_analytics")]
    #[prompt(
        name = "advanced_market_analysis",
        description = "Perform deep market microstructure analysis using order flow, volume profile, anomaly detection, and health scoring"
    )]
    pub async fn advanced_market_analysis(
        &self,
        Parameters(args): Parameters<AdvancedAnalysisArgs>,
    ) -> Result<GetPromptResult, ErrorData> {
        use crate::orderbook::analytics::{
            anomaly::detect_anomalies, flow::calculate_order_flow, health::calculate_health_score,
            profile::generate_volume_profile,
        };
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let symbol = &args.symbol;
        let storage = &self.snapshot_storage;

        // Determine time windows based on analysis depth
        let (flow_window, health_window, profile_hours) = match args.analysis_depth {
            Some(crate::server::types::AnalysisDepth::Quick) => (60, 300, 1), // 1min flow, 5min health, 1h profile
            Some(crate::server::types::AnalysisDepth::Deep) => (300, 1800, 24), // 5min flow, 30min health, 24h profile
            _ => (60, 300, 4), // Default: 1min flow, 5min health, 4h profile
        };

        // 1. Get order flow analysis
        let order_flow = calculate_order_flow(storage, symbol, flow_window, None)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to calculate order flow: {}", e), None)
            })?;

        // 2. Get volume profile (using 0.01 as default tick size for USDT pairs)
        let tick_size = Decimal::from_str("0.01").unwrap();
        let volume_profile = generate_volume_profile(symbol, profile_hours, tick_size)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to generate volume profile: {}", e), None)
            })?;

        // 3. Detect anomalies
        let anomalies = detect_anomalies(storage, symbol, flow_window)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to detect anomalies: {}", e), None)
            })?;

        // 4. Get market health score
        let health = calculate_health_score(storage, symbol, health_window)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to calculate health score: {}", e), None)
            })?;

        // Format comprehensive markdown response
        let mut content = format!(
            "# Advanced Market Analysis: {}\n\n\
            **Analysis Time**: {}\n\
            **Analysis Depth**: {:?}\n\n---\n\n",
            symbol,
            chrono::Utc::now().to_rfc3339(),
            args.analysis_depth
                .unwrap_or(crate::server::types::AnalysisDepth::Standard)
        );

        // Section 1: Order Flow Analysis
        content.push_str(&format!(
            "## 1. Order Flow Analysis (Last {} seconds)\n\n\
            **Flow Direction**: **{:?}** {}\n\
            - Bid Flow Rate: {:.2} orders/sec\n\
            - Ask Flow Rate: {:.2} orders/sec\n\
            - Net Flow: {:+.2} orders/sec\n\
            - Cumulative Delta: {:+.2}\n\n\
            *Interpretation*: {}\n\n---\n\n",
            flow_window,
            order_flow.flow_direction,
            match order_flow.flow_direction {
                FlowDirection::StrongBuy => "📈",
                FlowDirection::ModerateBuy => "↗️",
                FlowDirection::Neutral => "➡️",
                FlowDirection::ModerateSell => "↘️",
                FlowDirection::StrongSell => "📉",
            },
            order_flow.bid_flow_rate,
            order_flow.ask_flow_rate,
            order_flow.net_flow,
            order_flow.cumulative_delta,
            match order_flow.flow_direction {
                FlowDirection::StrongBuy =>
                    "Strong buying pressure with bid flow significantly higher than ask flow.",
                FlowDirection::ModerateBuy =>
                    "Moderate buying pressure. Bid flow exceeds ask flow.",
                FlowDirection::Neutral => "Balanced market. Bid and ask flows are roughly equal.",
                FlowDirection::ModerateSell =>
                    "Moderate selling pressure. Ask flow exceeds bid flow.",
                FlowDirection::StrongSell =>
                    "Strong selling pressure with ask flow significantly higher than bid flow.",
            }
        ));

        // Section 2: Volume Profile
        content.push_str(&format!(
            "## 2. Volume Profile (Last {} hours)\n\n\
            **Key Price Levels:**\n\
            - **POC (Point of Control)**: {}\n\
            - **VAH (Value Area High)**: {}\n\
            - **VAL (Value Area Low)**: {}\n\n\
            **Histogram**: {} price bins, {} total bin size\n\n\
            *Trading Strategy*: Price levels with high volume act as support/resistance. \
            POC represents fair value.\n\n---\n\n",
            profile_hours,
            volume_profile
                .poc_price
                .map(|p| format!("${}", p))
                .unwrap_or("N/A".to_string()),
            volume_profile
                .vah_price
                .map(|p| format!("${}", p))
                .unwrap_or("N/A".to_string()),
            volume_profile
                .val_price
                .map(|p| format!("${}", p))
                .unwrap_or("N/A".to_string()),
            volume_profile.histogram.len(),
            volume_profile.bin_size
        ));

        // Section 3: Market Health
        content.push_str(&format!(
            "## 3. Market Microstructure Health\n\n\
            **Overall Health Score**: **{:.0}/100** ({}) {}\n\n\
            **Component Breakdown:**\n\
            - Spread Stability: {:.0}/100\n\
            - Liquidity Depth: {:.0}/100\n\
            - Flow Balance: {:.0}/100\n\
            - Update Rate: {:.0}/100\n\n\
            **Trading Recommendation**: *{}*\n\n---\n\n",
            health.overall_score,
            health.health_level,
            match health.health_level.as_str() {
                "Excellent" => "✅",
                "Good" => "✅",
                "Fair" => "⚠️",
                "Poor" => "⚠️",
                "Critical" => "🔥",
                _ => "",
            },
            health.spread_stability_score,
            health.liquidity_depth_score,
            health.flow_balance_score,
            health.update_rate_score,
            health.recommended_action
        ));

        // Section 4: Anomaly Detection
        content.push_str(&format!(
            "## 4. Anomaly Detection\n\n\
            **Detected Anomalies**: {}\n\n",
            anomalies.len()
        ));

        if anomalies.is_empty() {
            content.push_str("**No anomalies detected** - Market conditions appear normal.\n\n");
        } else {
            for anomaly in &anomalies {
                let severity_emoji = match anomaly.severity {
                    crate::orderbook::analytics::types::Severity::Critical => "🔥",
                    crate::orderbook::analytics::types::Severity::High => "⚠️",
                    crate::orderbook::analytics::types::Severity::Medium => "⚠️",
                    crate::orderbook::analytics::types::Severity::Low => "ℹ️",
                };

                content.push_str(&format!(
                    "### {:?} ({:?}) {}\n\
                    - **Confidence**: {:.1}%\n\
                    - **Action**: {}\n\n",
                    anomaly.anomaly_type,
                    anomaly.severity,
                    severity_emoji,
                    anomaly.confidence * 100.0,
                    anomaly.recommended_action
                ));
            }
        }

        content.push_str("---\n\n");

        // Section 5: Summary
        content.push_str(&format!(
            "## 5. Summary & Recommendations\n\n\
            **Market Bias**: {:?}\n\
            **Risk Level**: {}\n\
            **Health Score**: {:.0}/100\n\
            **Anomalies**: {}\n\n\
            *Analysis generated using advanced orderbook analytics*\n\n\
            *Last updated: {}*\n",
            order_flow.flow_direction,
            health.health_level,
            health.overall_score,
            if anomalies.is_empty() {
                "None"
            } else {
                "Detected (see above)"
            },
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(GetPromptResult {
            description: Some("Comprehensive market microstructure analysis".to_string()),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
        })
    }

    /// Quick order flow snapshot prompt
    ///
    /// Provides instant order flow direction and bid/ask pressure for rapid trading decisions.
    #[cfg(feature = "orderbook_analytics")]
    #[prompt(
        name = "orderflow_snapshot",
        description = "Get instant order flow direction and bid/ask pressure for rapid trading decisions"
    )]
    pub async fn orderflow_snapshot(
        &self,
        Parameters(args): Parameters<OrderFlowSnapshotArgs>,
    ) -> Result<GetPromptResult, ErrorData> {
        use crate::orderbook::analytics::flow::calculate_order_flow;

        let symbol = &args.symbol;
        let window_secs = args.window_secs.unwrap_or(60).clamp(10, 300);
        let storage = &self.snapshot_storage;

        let order_flow = calculate_order_flow(storage, symbol, window_secs, None)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to calculate order flow: {}", e), None)
            })?;

        let content = format!(
            "# Order Flow Snapshot: {}\n\n\
            **Window**: Last {} seconds\n\
            **Timestamp**: {}\n\n\
            ## Flow Direction: **{:?}** {}\n\n\
            **Flow Metrics:**\n\
            - Bid Flow: {:.2} orders/sec\n\
            - Ask Flow: {:.2} orders/sec\n\
            - Net Flow: {:+.2} ({})\n\
            - Cumulative Delta: {:+.2}\n\n\
            **Quick Take**: {}\n\n\
            **Action**: {}\n",
            symbol,
            window_secs,
            chrono::Utc::now().to_rfc3339(),
            order_flow.flow_direction,
            match order_flow.flow_direction {
                FlowDirection::StrongBuy => "📈",
                FlowDirection::ModerateBuy => "↗️",
                FlowDirection::Neutral => "➡️",
                FlowDirection::ModerateSell => "↘️",
                FlowDirection::StrongSell => "📉",
            },
            order_flow.bid_flow_rate,
            order_flow.ask_flow_rate,
            order_flow.net_flow,
            if order_flow.net_flow > 0.0 {
                "buy pressure"
            } else {
                "sell pressure"
            },
            order_flow.cumulative_delta,
            match order_flow.flow_direction {
                FlowDirection::StrongBuy => format!(
                    "Aggressive buying. Bid flow {:.1}x stronger than ask flow.",
                    order_flow.bid_flow_rate / order_flow.ask_flow_rate.max(0.01)
                ),
                FlowDirection::ModerateBuy =>
                    "Moderate bullish momentum. More buyers than sellers.".to_string(),
                FlowDirection::Neutral => "Balanced market. No clear directional bias.".to_string(),
                FlowDirection::ModerateSell =>
                    "Moderate bearish momentum. More sellers than buyers.".to_string(),
                FlowDirection::StrongSell => format!(
                    "Aggressive selling. Ask flow {:.1}x stronger than bid flow.",
                    order_flow.ask_flow_rate / order_flow.bid_flow_rate.max(0.01)
                ),
            },
            match order_flow.flow_direction {
                FlowDirection::StrongBuy | FlowDirection::ModerateBuy =>
                    "Consider long entries if aligned with strategy. Strong demand evident.",
                FlowDirection::Neutral => "Wait for clearer direction. Market is balanced.",
                FlowDirection::ModerateSell | FlowDirection::StrongSell =>
                    "Consider short entries or exit longs. Supply pressure dominant.",
            }
        );

        Ok(GetPromptResult {
            description: Some("Real-time order flow snapshot".to_string()),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
        })
    }

    /// Market health check prompt
    ///
    /// Provides instant market health assessment before entering trades.
    #[cfg(feature = "orderbook_analytics")]
    #[prompt(
        name = "market_health_check",
        description = "Quick health check of market conditions before trading"
    )]
    pub async fn market_health_check(
        &self,
        Parameters(args): Parameters<MarketHealthCheckArgs>,
    ) -> Result<GetPromptResult, ErrorData> {
        use crate::orderbook::analytics::health::calculate_health_score;

        let symbol = &args.symbol;
        let storage = &self.snapshot_storage;

        let health = calculate_health_score(storage, symbol, 300)
            .await
            .map_err(|e| {
                ErrorData::internal_error(format!("Failed to calculate health score: {}", e), None)
            })?;

        let content = format!(
            "# Market Health: {}\n\n\
            **Overall Score**: **{:.0}/100** {} **{}**\n\n\
            **Status**: {}\n\n\
            **Breakdown:**\n\
            - {} Spread Stability: {:.0}/100\n\
            - {} Liquidity: {:.0}/100\n\
            - {} Flow Balance: {:.0}/100\n\
            - {} Activity: {:.0}/100\n\n\
            **Risk Assessment**: {}\n\n\
            **Recommendation**: {}\n\n\
            *Last updated: {}*\n",
            symbol,
            health.overall_score,
            match health.health_level.as_str() {
                "Excellent" => "✅",
                "Good" => "✅",
                "Fair" => "⚠️",
                "Poor" => "⚠️",
                "Critical" => "🔥",
                _ => "",
            },
            health.health_level,
            if health.overall_score >= 60.0 {
                "Safe to trade with normal position sizes"
            } else {
                "Exercise caution - market conditions deteriorating"
            },
            if health.spread_stability_score >= 70.0 {
                "✅"
            } else {
                "⚠️"
            },
            health.spread_stability_score,
            if health.liquidity_depth_score >= 70.0 {
                "✅"
            } else {
                "⚠️"
            },
            health.liquidity_depth_score,
            if health.flow_balance_score >= 70.0 {
                "✅"
            } else {
                "⚠️"
            },
            health.flow_balance_score,
            if health.update_rate_score >= 70.0 {
                "✅"
            } else {
                "⚠️"
            },
            health.update_rate_score,
            match health.overall_score {
                s if s >= 80.0 => "Low risk. Market conditions are optimal.",
                s if s >= 60.0 => "Low-medium risk. Normal trading conditions.",
                s if s >= 40.0 => "Medium risk. Exercise caution.",
                s if s >= 20.0 => "High risk. Reduce position sizes.",
                _ => "SEVERE RISK. Halt new trades immediately.",
            },
            health.recommended_action,
            chrono::Utc::now().to_rfc3339()
        );

        Ok(GetPromptResult {
            description: Some("Market health assessment".to_string()),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
        })
    }
}
