//! Tool Router Implementation
//!
//! Handles routing of MCP tool calls to appropriate handlers using rmcp macros.
//! Automatically generates JSON Schema for tool parameters and provides
//! structured routing for all Binance API tools.

use crate::server::BinanceServer;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{tool, tool_router, ErrorData};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[cfg(feature = "sse")]
use crate::tools::credentials::{validate_api_key, validate_api_secret};
#[cfg(feature = "sse")]
use crate::transport::sse::session::Credentials;
#[cfg(feature = "sse")]
use crate::types::Environment;

// Parameter structs for tools
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SymbolParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OrderBookParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
    /// Depth limit: 5, 10, 20, 50, 100, 500, 1000, 5000 (default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RecentTradesParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of trades to return (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct KlinesParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
    /// Interval: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
    pub interval: String,
    /// Number of klines (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// SSE version with session_id
#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountInfoParam {
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE version (no parameters needed)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountInfoParam {}

// SSE version with session_id
#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountTradesParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of trades (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE version (no session_id)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountTradesParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of trades (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// SSE version with session_id
#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PlaceOrderParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Order side: BUY or SELL
    pub side: String,
    /// Order type: LIMIT or MARKET
    #[serde(rename = "type")]
    pub order_type: String,
    /// Quantity to trade (e.g., 0.001)
    pub quantity: String,
    /// Price for LIMIT orders (required for LIMIT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE version (no session_id)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PlaceOrderParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Order side: BUY or SELL
    pub side: String,
    /// Order type: LIMIT or MARKET
    #[serde(rename = "type")]
    pub order_type: String,
    /// Quantity to trade (e.g., 0.001)
    pub quantity: String,
    /// Price for LIMIT orders (required for LIMIT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
}

// SSE version with session_id
#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OrderParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Order ID
    pub order_id: i64,
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE version (no session_id)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OrderParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Order ID
    pub order_id: i64,
}

// SSE version with session_id
#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenOrdersParam {
    /// Trading pair (optional, returns all if omitted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE version (no session_id)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenOrdersParam {
    /// Trading pair (optional, returns all if omitted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

// SSE version with session_id
#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AllOrdersParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of orders (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE version (no session_id)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AllOrdersParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of orders (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[cfg(feature = "sse")]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ConfigureCredentialsParam {
    /// Binance API key (exactly 64 alphanumeric characters)
    pub api_key: String,
    /// Binance API secret (exactly 64 alphanumeric characters)
    pub api_secret: String,
    /// Target environment: "testnet" or "mainnet"
    pub environment: String,
    /// Session ID from Mcp-Session-Id header
    pub session_id: String,
}

// Non-SSE stub version (credentials not supported)
#[cfg(not(feature = "sse"))]
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ConfigureCredentialsParam {}

#[tool_router(vis = "pub")]
impl BinanceServer {
    /// Get current Binance server time
    ///
    /// Returns the current server time in milliseconds since Unix epoch.
    /// Useful for time synchronization and validating server connectivity.
    ///
    /// # Returns
    /// JSON object with:
    /// - `serverTime`: Server timestamp in milliseconds
    /// - `offset`: Time difference between server and local time
    #[tool(
        description = "Returns current Binance server time in milliseconds since Unix epoch. Useful for time synchronization and connectivity validation."
    )]
    pub async fn get_server_time(&self) -> Result<CallToolResult, ErrorData> {
        // Get local time before API call
        let local_time_before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ErrorData::internal_error(format!("System time error: {}", e), None))?
            .as_millis() as i64;

        // Call Binance API
        let server_time = self
            .binance_client
            .get_server_time()
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        // Calculate offset
        let local_time_after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ErrorData::internal_error(format!("System time error: {}", e), None))?
            .as_millis() as i64;

        let local_time_avg = (local_time_before + local_time_after) / 2;
        let offset = server_time - local_time_avg;

        // Log time synchronization info
        tracing::info!(
            "Binance server time: {} (offset: {}ms)",
            server_time,
            offset
        );

        // Warn if offset is significant (>5 seconds)
        if offset.abs() > 5000 {
            tracing::warn!(
                "Large time offset detected: {}ms. Consider syncing system clock.",
                offset
            );
        }

        // Create response JSON
        let response_json = json!({
            "serverTime": server_time,
            "offset": offset
        });

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Configure API credentials for session (SSE feature only)
    ///
    /// Stores Binance API credentials (testnet or mainnet) for this session.
    /// Credentials are validated synchronously (<10ms) and stored in memory only.
    /// Never persisted to disk. Automatically cleared when session ends.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Binance API key (exactly 64 alphanumeric characters)
    /// * `api_secret` - Binance API secret (exactly 64 alphanumeric characters)
    /// * `environment` - Target environment ("testnet" or "mainnet")
    /// * `session_id` - Session ID from Mcp-Session-Id header
    ///
    /// # Returns
    ///
    /// Success response with configuration details or structured error.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - API key format invalid (not 64 alphanumeric chars)
    /// - API secret format invalid (not 64 alphanumeric chars)
    /// - Environment invalid (not "testnet" or "mainnet")
    /// - Session not found
    #[cfg(feature = "sse")]
    #[tool(
        description = "Configure Binance API credentials for this session. Supports testnet and mainnet. Credentials validated (<10ms) and stored in memory only (never persisted to disk)."
    )]
    pub async fn configure_credentials(
        &self,
        params: Parameters<ConfigureCredentialsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;

        // Validate API key format (synchronous, <10ms)
        if let Err(e) = validate_api_key(&p.api_key) {
            let error_json = json!({
                "error_code": "INVALID_API_KEY_FORMAT",
                "message": e.to_string(),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        // Validate API secret format (synchronous, <10ms)
        if let Err(e) = validate_api_secret(&p.api_secret) {
            let error_json = json!({
                "error_code": "INVALID_API_SECRET_FORMAT",
                "message": e.to_string(),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        // Parse environment
        let environment = match Environment::from_str(&p.environment) {
            Ok(env) => env,
            Err(msg) => {
                let error_json = json!({
                    "error_code": "INVALID_ENVIRONMENT",
                    "message": msg,
                });
                return Ok(CallToolResult::success(vec![Content::text(
                    error_json.to_string(),
                )]));
            }
        };

        // Create credentials struct
        let credentials = Credentials::new(
            p.api_key.clone(),
            p.api_secret,
            environment,
            p.session_id.clone(),
        );

        // Store credentials in session manager
        let stored = self.session_manager.store_credentials(credentials).await;

        if !stored {
            let error_json = json!({
                "error_code": "SESSION_NOT_FOUND",
                "message": format!("Session {} not found. Ensure SSE connection is active.", p.session_id),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        // Log successful configuration (mask API key, never log secret)
        let key_prefix: String = p.api_key.chars().take(8).collect();
        tracing::info!(
            session_id = %p.session_id,
            environment = %environment,
            key_prefix = %key_prefix,
            "API credentials configured for session"
        );

        // Return success response
        let response_json = json!({
            "configured": true,
            "environment": environment.to_string(),
            "key_prefix": key_prefix,
            "message": format!("Credentials successfully configured for {} environment", environment),
        });

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Stub implementation for configure_credentials when SSE feature is disabled
    #[cfg(not(feature = "sse"))]
    #[tool(description = "Credential management not available (requires 'sse' feature)")]
    pub async fn configure_credentials(
        &self,
        _params: Parameters<ConfigureCredentialsParam>,
    ) -> Result<CallToolResult, ErrorData> {
        Err(ErrorData::internal_error(
            "Credential management is not enabled in this deployment. Rebuild with --features sse"
                .to_string(),
            None,
        ))
    }

    /// Get 24-hour ticker price change statistics
    ///
    /// Returns price change statistics for the last 24 hours for a trading pair.
    #[tool(
        description = "Get 24-hour ticker price change statistics for a symbol. Returns price, volume, high, low, and change percentage."
    )]
    pub async fn get_ticker(
        &self,
        params: Parameters<SymbolParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let ticker = self
            .binance_client
            .get_24hr_ticker(&params.0.symbol)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&ticker)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get order book depth
    ///
    /// Returns current order book with bids and asks for a trading pair.
    #[tool(
        description = "Get current order book depth (bids and asks) for a symbol. Returns price levels and quantities."
    )]
    pub async fn get_order_book(
        &self,
        params: Parameters<OrderBookParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let order_book = self
            .binance_client
            .get_order_book(&params.0.symbol, params.0.limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order_book)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get recent trades
    ///
    /// Returns list of recent trades for a trading pair.
    #[tool(
        description = "Get recent public trades for a symbol. Returns trade history with prices, quantities, and timestamps."
    )]
    pub async fn get_recent_trades(
        &self,
        params: Parameters<RecentTradesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let trades = self
            .binance_client
            .get_recent_trades(&params.0.symbol, params.0.limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&trades)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get candlestick/kline data
    ///
    /// Returns OHLCV (Open, High, Low, Close, Volume) candlestick data.
    #[tool(
        description = "Get candlestick/kline data (OHLCV) for technical analysis. Supports multiple timeframes from 1m to 1M."
    )]
    pub async fn get_klines(
        &self,
        params: Parameters<KlinesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let klines = self
            .binance_client
            .get_klines(&params.0.symbol, &params.0.interval, params.0.limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&klines)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get current average price
    ///
    /// Returns current average price for a symbol.
    #[tool(
        description = "Get current average price for a symbol. Simpler alternative to 24hr ticker."
    )]
    pub async fn get_average_price(
        &self,
        params: Parameters<SymbolParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let price = self
            .binance_client
            .get_ticker_price(&params.0.symbol)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&price)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get account information (SSE version with session credentials)
    ///
    /// Returns account balances and trading permissions. Requires API credentials.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Get account information including balances and permissions. Requires API credentials configured via configure_credentials."
    )]
    pub async fn get_account_info(
        &self,
        params: Parameters<AccountInfoParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let account = self
            .binance_client
            .get_account(credentials.as_ref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&account)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get account information (non-SSE version with environment credentials)
    ///
    /// Returns account balances and trading permissions. Requires API credentials.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Get account information including balances and permissions. Requires API credentials."
    )]
    pub async fn get_account_info(&self) -> Result<CallToolResult, ErrorData> {
        let account = self
            .binance_client
            .get_account()
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&account)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get account trade history (SSE version with session credentials)
    ///
    /// Returns trade history for your account on a specific symbol. Requires API credentials.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Get your account trade history for a symbol. Returns executed trades with fees and commissions. Requires API credentials configured via configure_credentials."
    )]
    pub async fn get_account_trades(
        &self,
        params: Parameters<AccountTradesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let trades = self
            .binance_client
            .get_my_trades(&params.0.symbol, params.0.limit, credentials.as_ref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&trades)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get account trade history (non-SSE version with environment credentials)
    ///
    /// Returns trade history for your account on a specific symbol. Requires API credentials.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Get your account trade history for a symbol. Returns executed trades with fees and commissions. Requires API credentials."
    )]
    pub async fn get_account_trades(
        &self,
        params: Parameters<AccountTradesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let trades = self
            .binance_client
            .get_my_trades(&params.0.symbol, params.0.limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&trades)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Place a new order (SSE version with session credentials)
    ///
    /// Creates a new trading order. Requires API credentials.
    /// ⚠️ TESTNET ONLY - Use testnet credentials to avoid real trades.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Place a new order (BUY/SELL, LIMIT/MARKET). ⚠️ Use TESTNET credentials only! Requires API credentials configured via configure_credentials."
    )]
    pub async fn place_order(
        &self,
        params: Parameters<PlaceOrderParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let order = self
            .binance_client
            .create_order(
                &params.0.symbol,
                &params.0.side,
                &params.0.order_type,
                &params.0.quantity,
                params.0.price.as_deref(),
                credentials.as_ref(),
            )
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Place a new order (non-SSE version with environment credentials)
    ///
    /// Creates a new trading order. Requires API credentials.
    /// ⚠️ TESTNET ONLY - Use testnet credentials to avoid real trades.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Place a new order (BUY/SELL, LIMIT/MARKET). ⚠️ Use TESTNET credentials only! Requires API credentials."
    )]
    pub async fn place_order(
        &self,
        params: Parameters<PlaceOrderParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let order = self
            .binance_client
            .create_order(
                &params.0.symbol,
                &params.0.side,
                &params.0.order_type,
                &params.0.quantity,
                params.0.price.as_deref(),
            )
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Query order status (SSE version with session credentials)
    ///
    /// Get details of a specific order by orderId. Requires API credentials.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Query the status of a specific order by orderId. Returns current order state. Requires API credentials configured via configure_credentials."
    )]
    pub async fn get_order(
        &self,
        params: Parameters<OrderParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let order = self
            .binance_client
            .query_order(&params.0.symbol, params.0.order_id, credentials.as_ref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Query order status (non-SSE version with environment credentials)
    ///
    /// Get details of a specific order by orderId. Requires API credentials.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Query the status of a specific order by orderId. Returns current order state. Requires API credentials."
    )]
    pub async fn get_order(
        &self,
        params: Parameters<OrderParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let order = self
            .binance_client
            .query_order(&params.0.symbol, params.0.order_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Cancel an order (SSE version with session credentials)
    ///
    /// Cancel an active order. Requires API credentials.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Cancel an active order by orderId. Returns canceled order details. Requires API credentials configured via configure_credentials."
    )]
    pub async fn cancel_order(
        &self,
        params: Parameters<OrderParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let order = self
            .binance_client
            .cancel_order(&params.0.symbol, params.0.order_id, credentials.as_ref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Cancel an order (non-SSE version with environment credentials)
    ///
    /// Cancel an active order. Requires API credentials.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Cancel an active order by orderId. Returns canceled order details. Requires API credentials."
    )]
    pub async fn cancel_order(
        &self,
        params: Parameters<OrderParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let order = self
            .binance_client
            .cancel_order(&params.0.symbol, params.0.order_id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&order)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get all open orders (SSE version with session credentials)
    ///
    /// Returns all currently active orders. Requires API credentials.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Get all open orders. Optionally filter by symbol or get all open orders across all pairs. Requires API credentials configured via configure_credentials."
    )]
    pub async fn get_open_orders(
        &self,
        params: Parameters<OpenOrdersParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let orders = self
            .binance_client
            .get_open_orders(params.0.symbol.as_deref(), credentials.as_ref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&orders)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get all open orders (non-SSE version with environment credentials)
    ///
    /// Returns all currently active orders. Requires API credentials.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Get all open orders. Optionally filter by symbol or get all open orders across all pairs. Requires API credentials."
    )]
    pub async fn get_open_orders(
        &self,
        params: Parameters<OpenOrdersParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let orders = self
            .binance_client
            .get_open_orders(params.0.symbol.as_deref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&orders)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get all orders (history) (SSE version with session credentials)
    ///
    /// Returns all orders (active, canceled, filled) for a symbol. Requires API credentials.
    #[cfg(feature = "sse")]
    #[tool(
        description = "Get complete order history for a symbol (active, canceled, filled). Requires API credentials configured via configure_credentials."
    )]
    pub async fn get_all_orders(
        &self,
        params: Parameters<AllOrdersParam>,
    ) -> Result<CallToolResult, ErrorData> {
        // Retrieve credentials from session
        let credentials = self
            .session_manager
            .get_credentials(&params.0.session_id)
            .await;

        if credentials.is_none() {
            let error_json = json!({
                "error_code": "CREDENTIALS_NOT_CONFIGURED",
                "message": "API credentials not configured for this session. Call configure_credentials first."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                error_json.to_string(),
            )]));
        }

        let orders = self
            .binance_client
            .get_all_orders(&params.0.symbol, params.0.limit, credentials.as_ref())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&orders)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get all orders (history) (non-SSE version with environment credentials)
    ///
    /// Returns all orders (active, canceled, filled) for a symbol. Requires API credentials.
    #[cfg(not(feature = "sse"))]
    #[tool(
        description = "Get complete order history for a symbol (active, canceled, filled). Requires API credentials."
    )]
    pub async fn get_all_orders(
        &self,
        params: Parameters<AllOrdersParam>,
    ) -> Result<CallToolResult, ErrorData> {
        let orders = self
            .binance_client
            .get_all_orders(&params.0.symbol, params.0.limit)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&orders)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get L1 aggregated metrics for quick spread assessment
    ///
    /// Provides lightweight order book analysis (15% token cost vs L2-full):
    /// - Spread in basis points
    /// - Microprice (volume-weighted fair price)
    /// - Bid/ask volume imbalance
    /// - Wall detection (large levels)
    /// - VWAP-based slippage estimates
    ///
    /// First request: 2-3s (lazy initialization). Subsequent: <200ms (cached).
    #[cfg(feature = "orderbook")]
    #[tool(
        description = "Get L1 aggregated order book metrics for quick spread assessment. Includes spread, microprice, imbalance, walls, and slippage estimates. Lightweight (15% token cost vs full depth)."
    )]
    pub async fn get_orderbook_metrics(
        &self,
        params: Parameters<crate::orderbook::tools::GetOrderBookMetricsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let metrics = crate::orderbook::tools::get_orderbook_metrics(
            self.orderbook_manager.clone(),
            params.0,
        )
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&metrics)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get L2 depth with compact integer encoding
    ///
    /// Token cost: 50% (L2-lite with 20 levels) or 100% (L2-full with 100 levels).
    ///
    /// Compact encoding reduces JSON size by ~40%:
    /// - price_scale = 100 (e.g., 67650.00 → 6765000)
    /// - qty_scale = 100000 (e.g., 1.234 → 123400)
    ///
    /// First request: 2-3s (lazy initialization). Subsequent: <300ms (cached).
    #[cfg(feature = "orderbook")]
    #[tool(
        description = "Get L2 order book depth with compact integer encoding. Returns price levels and quantities. Use levels=20 for L2-lite (50% cost) or levels=100 for L2-full (100% cost)."
    )]
    pub async fn get_orderbook_depth(
        &self,
        params: Parameters<crate::orderbook::tools::GetOrderBookDepthParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let depth =
            crate::orderbook::tools::get_orderbook_depth(self.orderbook_manager.clone(), params.0)
                .await
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&depth)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Get service health status for order book tracking
    ///
    /// Returns operational visibility:
    /// - Overall status (ok/degraded/error)
    /// - Number of active symbol subscriptions (0-20)
    /// - Data freshness (last update age in ms)
    /// - WebSocket connection status
    ///
    /// Latency: <50ms (no external API calls).
    #[cfg(feature = "orderbook")]
    #[tool(
        description = "Get order book service health status. Returns connection status, active symbols (0-20), and data freshness. Fast (<50ms, no API calls)."
    )]
    pub async fn get_orderbook_health(&self) -> Result<CallToolResult, ErrorData> {
        let health = crate::orderbook::tools::get_orderbook_health(self.orderbook_manager.clone())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let response_json = serde_json::to_value(&health)
            .map_err(|e| ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    /// Stub implementation for get_orderbook_metrics when orderbook feature is disabled
    #[cfg(not(feature = "orderbook"))]
    #[tool(description = "Order book metrics not available (requires 'orderbook' feature)")]
    pub async fn get_orderbook_metrics(
        &self,
        _params: Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        Err(ErrorData::internal_error(
            "Order book features are not enabled in this deployment. Rebuild with --features orderbook".to_string(),
            None,
        ))
    }

    /// Stub implementation for get_orderbook_depth when orderbook feature is disabled
    #[cfg(not(feature = "orderbook"))]
    #[tool(description = "Order book depth not available (requires 'orderbook' feature)")]
    pub async fn get_orderbook_depth(
        &self,
        _params: Parameters<serde_json::Value>,
    ) -> Result<CallToolResult, ErrorData> {
        Err(ErrorData::internal_error(
            "Order book features are not enabled in this deployment. Rebuild with --features orderbook".to_string(),
            None,
        ))
    }

    /// Stub implementation for get_orderbook_health when orderbook feature is disabled
    #[cfg(not(feature = "orderbook"))]
    #[tool(description = "Order book health not available (requires 'orderbook' feature)")]
    pub async fn get_orderbook_health(&self) -> Result<CallToolResult, ErrorData> {
        Err(ErrorData::internal_error(
            "Order book features are not enabled in this deployment. Rebuild with --features orderbook".to_string(),
            None,
        ))
    }
}
