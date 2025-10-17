//! Tool Router Implementation
//!
//! Handles routing of MCP tool calls to appropriate handlers using rmcp macros.
//! Automatically generates JSON Schema for tool parameters and provides
//! structured routing for all Binance API tools.

use crate::server::BinanceServer;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{ErrorData, tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountTradesParam {
    /// Trading pair symbol (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of trades (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OrderParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Order ID
    pub order_id: i64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OpenOrdersParam {
    /// Trading pair (optional, returns all if omitted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AllOrdersParam {
    /// Trading pair (e.g., BTCUSDT)
    pub symbol: String,
    /// Number of orders (default: 500, max: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

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

    /// Get account information
    ///
    /// Returns account balances and trading permissions. Requires API credentials.
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

    /// Get account trade history
    ///
    /// Returns trade history for your account on a specific symbol. Requires API credentials.
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

    /// Place a new order
    ///
    /// Creates a new trading order. Requires API credentials.
    /// ⚠️ TESTNET ONLY - Use testnet credentials to avoid real trades.
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

    /// Query order status
    ///
    /// Get details of a specific order by orderId. Requires API credentials.
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

    /// Cancel an order
    ///
    /// Cancel an active order. Requires API credentials.
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

    /// Get all open orders
    ///
    /// Returns all currently active orders. Requires API credentials.
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

    /// Get all orders (history)
    ///
    /// Returns all orders (active, canceled, filled) for a symbol. Requires API credentials.
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
}
