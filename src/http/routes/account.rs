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
    if let Some(limit) = params.limit
        && limit > 1000
    {
        return Err(McpError::InvalidRequest(
            "limit cannot exceed 1000".to_string(),
        ));
    }

    let trades = state
        .binance_client
        .get_my_trades(&params.symbol, params.limit)
        .await?;

    Ok(Json(serde_json::to_value(trades)?))
}

/// Response for listen key creation
#[derive(Debug, serde::Serialize)]
pub struct ListenKeyResponse {
    /// Listen key for user data stream
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

/// POST /api/v1/userDataStream - Create a listen key for user data stream
///
/// Creates a listen key valid for 60 minutes. The key must be kept alive
/// by calling PUT /api/v1/userDataStream every 30 minutes.
///
/// Requires API key to be configured.
///
/// ## Example
/// ```bash
/// curl -X POST -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/userDataStream'
/// ```
///
/// ## Response
/// ```json
/// {
///   "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
/// }
/// ```
pub async fn create_user_data_stream(
    State(state): State<AppState>,
) -> Result<Json<ListenKeyResponse>, McpError> {
    tracing::info!("POST /api/v1/userDataStream");

    let listen_key = state.binance_client.create_listen_key().await?;

    Ok(Json(ListenKeyResponse { listen_key }))
}

/// PUT /api/v1/userDataStream - Keep alive a listen key
///
/// Extends the validity of the listen key by 60 minutes from the current time.
/// Must be called at least once every 30 minutes to prevent expiration.
///
/// Requires API key to be configured.
///
/// ## Example
/// ```bash
/// curl -X PUT -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/userDataStream?listenKey=YOUR_LISTEN_KEY'
/// ```
///
/// ## Response
/// ```json
/// {}
/// ```
#[derive(Debug, Deserialize)]
pub struct KeepAliveQuery {
    /// Listen key to keep alive
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

pub async fn keepalive_user_data_stream(
    State(state): State<AppState>,
    Query(params): Query<KeepAliveQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!("PUT /api/v1/userDataStream listenKey={}", params.listen_key);

    state
        .binance_client
        .keepalive_listen_key(&params.listen_key)
        .await?;

    Ok(Json(serde_json::json!({})))
}

/// DELETE /api/v1/userDataStream - Close a listen key
///
/// Closes the user data stream and invalidates the listen key immediately.
///
/// Requires API key to be configured.
///
/// ## Example
/// ```bash
/// curl -X DELETE -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/userDataStream?listenKey=YOUR_LISTEN_KEY'
/// ```
///
/// ## Response
/// ```json
/// {}
/// ```
#[derive(Debug, Deserialize)]
pub struct CloseQuery {
    /// Listen key to close
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

pub async fn close_user_data_stream(
    State(state): State<AppState>,
    Query(params): Query<CloseQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "DELETE /api/v1/userDataStream listenKey={}",
        params.listen_key
    );

    state
        .binance_client
        .close_listen_key(&params.listen_key)
        .await?;

    Ok(Json(serde_json::json!({})))
}
