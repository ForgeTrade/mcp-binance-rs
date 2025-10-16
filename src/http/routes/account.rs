//! Account Information REST API Endpoints
//!
//! Provides HTTP endpoints for querying account information:
//! - GET /api/v1/account - Get account information and balances
//! - GET /api/v1/myTrades - Get trade history for a symbol

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::error::McpError;
use crate::http::AppState;

/// Query parameters for my trades endpoint
#[derive(Debug, Deserialize)]
pub struct MyTradesQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Number of trades to return (max 1000)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// GET /api/v1/account - Get account information and balances
///
/// Returns account information including balances, permissions, and commission rates.
/// Requires API key and secret to be configured.
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/account'
/// ```
///
/// ## Response
/// ```json
/// {
///   "makerCommission": 10,
///   "takerCommission": 10,
///   "buyerCommission": 0,
///   "sellerCommission": 0,
///   "canTrade": true,
///   "canWithdraw": true,
///   "canDeposit": true,
///   "updateTime": 1699564800000,
///   "accountType": "SPOT",
///   "balances": [
///     {
///       "asset": "BTC",
///       "free": "0.00100000",
///       "locked": "0.00000000"
///     },
///     ...
///   ],
///   "permissions": ["SPOT"]
/// }
/// ```
pub async fn get_account(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!("GET /api/v1/account");

    let account = state.binance_client.get_account().await?;

    Ok(Json(serde_json::to_value(account)?))
}

/// GET /api/v1/myTrades - Get trade history for a symbol
///
/// Returns historical trades for the authenticated account on a specific symbol.
/// Requires API key and secret to be configured.
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/myTrades?symbol=BTCUSDT&limit=100'
/// ```
///
/// ## Response
/// ```json
/// [
///   {
///     "symbol": "BTCUSDT",
///     "id": 28457,
///     "orderId": 100234,
///     "price": "50000.00",
///     "qty": "0.001",
///     "quoteQty": "50.00",
///     "commission": "0.05",
///     "commissionAsset": "USDT",
///     "time": 1699564800000,
///     "isBuyer": true,
///     "isMaker": false,
///     "isBestMatch": true
///   },
///   ...
/// ]
/// ```
pub async fn get_my_trades(
    State(state): State<AppState>,
    Query(params): Query<MyTradesQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "GET /api/v1/myTrades symbol={} limit={:?}",
        params.symbol,
        params.limit
    );

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest("symbol is required".to_string()));
    }

    // Validate limit
    if let Some(limit) = params.limit {
        if limit > 1000 {
            return Err(McpError::InvalidRequest(
                "limit cannot exceed 1000".to_string(),
            ));
        }
    }

    let trades = state
        .binance_client
        .get_my_trades(&params.symbol, params.limit)
        .await?;

    Ok(Json(serde_json::to_value(trades)?))
}
