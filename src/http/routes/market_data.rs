//! Market Data REST API Endpoints
//!
//! Provides HTTP endpoints for querying Binance market data:
//! - GET /api/v1/ticker/price - Latest price for a symbol
//! - GET /api/v1/ticker/24hr - 24-hour statistics
//! - GET /api/v1/klines - Candlestick data
//! - GET /api/v1/depth - Order book depth
//! - GET /api/v1/trades - Recent trades

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::McpError;
use crate::http::AppState;

/// Query parameters for ticker price endpoint
#[derive(Debug, Deserialize)]
pub struct TickerPriceQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
}

/// Query parameters for 24hr ticker endpoint
#[derive(Debug, Deserialize)]
pub struct Ticker24hrQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
}

/// Query parameters for klines endpoint
#[derive(Debug, Deserialize)]
pub struct KlinesQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Kline interval (e.g., "1m", "5m", "1h", "1d")
    pub interval: String,
    /// Number of klines to return (default 500, max 1000)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Query parameters for order book depth endpoint
#[derive(Debug, Deserialize)]
pub struct DepthQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Number of levels to return (valid: 5, 10, 20, 50, 100, 500, 1000, 5000)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Query parameters for recent trades endpoint
#[derive(Debug, Deserialize)]
pub struct TradesQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Number of trades to return (default 500, max 1000)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// GET /api/v1/ticker/price - Get latest price for a symbol
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/ticker/price?symbol=BTCUSDT'
/// ```
pub async fn get_ticker_price(
    State(state): State<AppState>,
    Query(params): Query<TickerPriceQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!("GET /api/v1/ticker/price symbol={}", params.symbol);

    // Validate symbol parameter
    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest(
            "symbol parameter is required".to_string(),
        ));
    }

    // Call Binance API
    let ticker = state
        .binance_client
        .get_ticker_price(&params.symbol)
        .await?;

    Ok(Json(serde_json::to_value(ticker)?))
}

/// GET /api/v1/ticker/24hr - Get 24-hour ticker statistics
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/ticker/24hr?symbol=BTCUSDT'
/// ```
pub async fn get_ticker_24hr(
    State(state): State<AppState>,
    Query(params): Query<Ticker24hrQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!("GET /api/v1/ticker/24hr symbol={}", params.symbol);

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest(
            "symbol parameter is required".to_string(),
        ));
    }

    let ticker = state.binance_client.get_24hr_ticker(&params.symbol).await?;

    Ok(Json(serde_json::to_value(ticker)?))
}

/// GET /api/v1/klines - Get candlestick data
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/klines?symbol=BTCUSDT&interval=1h&limit=100'
/// ```
pub async fn get_klines(
    State(state): State<AppState>,
    Query(params): Query<KlinesQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "GET /api/v1/klines symbol={} interval={} limit={:?}",
        params.symbol,
        params.interval,
        params.limit
    );

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest(
            "symbol parameter is required".to_string(),
        ));
    }

    if params.interval.is_empty() {
        return Err(McpError::InvalidRequest(
            "interval parameter is required".to_string(),
        ));
    }

    // Validate limit
    if let Some(limit) = params.limit {
        if limit > 1000 {
            return Err(McpError::InvalidRequest(
                "limit cannot exceed 1000".to_string(),
            ));
        }
    }

    let klines = state
        .binance_client
        .get_klines(&params.symbol, &params.interval, params.limit)
        .await?;

    Ok(Json(serde_json::to_value(klines)?))
}

/// GET /api/v1/depth - Get order book depth
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/depth?symbol=BTCUSDT&limit=10'
/// ```
pub async fn get_depth(
    State(state): State<AppState>,
    Query(params): Query<DepthQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "GET /api/v1/depth symbol={} limit={:?}",
        params.symbol,
        params.limit
    );

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest(
            "symbol parameter is required".to_string(),
        ));
    }

    // Validate limit is one of the allowed values
    if let Some(limit) = params.limit {
        let valid_limits = [5, 10, 20, 50, 100, 500, 1000, 5000];
        if !valid_limits.contains(&limit) {
            return Err(McpError::InvalidRequest(format!(
                "limit must be one of: {:?}",
                valid_limits
            )));
        }
    }

    let order_book = state
        .binance_client
        .get_order_book(&params.symbol, params.limit)
        .await?;

    Ok(Json(serde_json::to_value(order_book)?))
}

/// GET /api/v1/trades - Get recent trades
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/trades?symbol=BTCUSDT&limit=100'
/// ```
pub async fn get_trades(
    State(state): State<AppState>,
    Query(params): Query<TradesQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "GET /api/v1/trades symbol={} limit={:?}",
        params.symbol,
        params.limit
    );

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest(
            "symbol parameter is required".to_string(),
        ));
    }

    if let Some(limit) = params.limit {
        if limit > 1000 {
            return Err(McpError::InvalidRequest(
                "limit cannot exceed 1000".to_string(),
            ));
        }
    }

    let trades = state
        .binance_client
        .get_recent_trades(&params.symbol, params.limit)
        .await?;

    Ok(Json(serde_json::to_value(trades)?))
}
