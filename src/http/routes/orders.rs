//! Order Management REST API Endpoints
//!
//! Provides HTTP endpoints for placing and managing orders:
//! - POST /api/v1/order - Create new order
//! - DELETE /api/v1/order - Cancel existing order
//! - GET /api/v1/order - Query order status
//! - GET /api/v1/openOrders - Get all open orders
//! - GET /api/v1/allOrders - Get all orders (filled, canceled, etc.)

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::McpError;
use crate::http::AppState;

/// Request body for order creation endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Order side: "BUY" or "SELL"
    pub side: String,
    /// Order type: "LIMIT", "MARKET", "STOP_LOSS", etc.
    #[serde(rename = "type")]
    pub order_type: String,
    /// Order quantity
    pub quantity: String,
    /// Price (required for LIMIT orders)
    pub price: Option<String>,
}

/// Query parameters for cancel order endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Order ID to cancel
    pub order_id: i64,
}

/// Query parameters for query order endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Order ID to query
    pub order_id: i64,
}

/// Query parameters for open orders endpoint
#[derive(Debug, Deserialize)]
pub struct OpenOrdersQuery {
    /// Optional symbol filter (e.g., "BTCUSDT")
    pub symbol: Option<String>,
}

/// Query parameters for all orders endpoint
#[derive(Debug, Deserialize)]
pub struct AllOrdersQuery {
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Number of orders to return (max 1000)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// POST /api/v1/order - Create a new order
///
/// ## Example
/// ```bash
/// curl -X POST -H "Authorization: Bearer token" \
///   -H "Content-Type: application/json" \
///   -d '{"symbol":"BTCUSDT","side":"BUY","type":"LIMIT","quantity":"0.001","price":"50000"}' \
///   'http://localhost:8080/api/v1/order'
/// ```
pub async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "POST /api/v1/order symbol={} side={} type={} quantity={} price={:?}",
        req.symbol,
        req.side,
        req.order_type,
        req.quantity,
        req.price
    );

    // Validate required parameters
    if req.symbol.is_empty() {
        return Err(McpError::InvalidRequest("symbol is required".to_string()));
    }
    if req.side.is_empty() {
        return Err(McpError::InvalidRequest("side is required".to_string()));
    }
    if req.order_type.is_empty() {
        return Err(McpError::InvalidRequest("type is required".to_string()));
    }
    if req.quantity.is_empty() {
        return Err(McpError::InvalidRequest("quantity is required".to_string()));
    }

    // Validate side
    if req.side != "BUY" && req.side != "SELL" {
        return Err(McpError::InvalidRequest(
            "side must be 'BUY' or 'SELL'".to_string(),
        ));
    }

    // Validate LIMIT orders have price
    if req.order_type == "LIMIT" && req.price.is_none() {
        return Err(McpError::InvalidRequest(
            "price is required for LIMIT orders".to_string(),
        ));
    }

    // Call Binance API
    let order = state
        .binance_client
        .create_order(
            &req.symbol,
            &req.side,
            &req.order_type,
            &req.quantity,
            req.price.as_deref(),
            None,
        )
        .await?;

    Ok(Json(serde_json::to_value(order)?))
}

/// DELETE /api/v1/order - Cancel an existing order
///
/// ## Example
/// ```bash
/// curl -X DELETE -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/order?symbol=BTCUSDT&orderId=12345678'
/// ```
pub async fn cancel_order(
    State(state): State<AppState>,
    Query(params): Query<CancelOrderQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "DELETE /api/v1/order symbol={} orderId={}",
        params.symbol,
        params.order_id
    );

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest("symbol is required".to_string()));
    }

    let order = state
        .binance_client
        .cancel_order(&params.symbol, params.order_id, None)
        .await?;

    Ok(Json(serde_json::to_value(order)?))
}

/// GET /api/v1/order - Query order status
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/order?symbol=BTCUSDT&orderId=12345678'
/// ```
pub async fn query_order(
    State(state): State<AppState>,
    Query(params): Query<QueryOrderQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "GET /api/v1/order symbol={} orderId={}",
        params.symbol,
        params.order_id
    );

    if params.symbol.is_empty() {
        return Err(McpError::InvalidRequest("symbol is required".to_string()));
    }

    let order = state
        .binance_client
        .query_order(&params.symbol, params.order_id, None)
        .await?;

    Ok(Json(serde_json::to_value(order)?))
}

/// GET /api/v1/openOrders - Get all open orders
///
/// ## Example
/// ```bash
/// # All open orders
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/openOrders'
///
/// # Open orders for specific symbol
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/openOrders?symbol=BTCUSDT'
/// ```
pub async fn get_open_orders(
    State(state): State<AppState>,
    Query(params): Query<OpenOrdersQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!("GET /api/v1/openOrders symbol={:?}", params.symbol);

    let orders = state
        .binance_client
        .get_open_orders(params.symbol.as_deref(), None)
        .await?;

    Ok(Json(serde_json::to_value(orders)?))
}

/// GET /api/v1/allOrders - Get all orders (filled, canceled, etc.)
///
/// ## Example
/// ```bash
/// curl -H "Authorization: Bearer token" \
///   'http://localhost:8080/api/v1/allOrders?symbol=BTCUSDT&limit=100'
/// ```
pub async fn get_all_orders(
    State(state): State<AppState>,
    Query(params): Query<AllOrdersQuery>,
) -> Result<Json<serde_json::Value>, McpError> {
    tracing::info!(
        "GET /api/v1/allOrders symbol={} limit={:?}",
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

    let orders = state
        .binance_client
        .get_all_orders(&params.symbol, params.limit, None)
        .await?;

    Ok(Json(serde_json::to_value(orders)?))
}
