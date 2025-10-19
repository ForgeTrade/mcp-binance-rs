//! Simplified SSE handlers for MVP (T020-T021)
//!
//! This is a minimal implementation to get tests passing quickly.
//! Will be enhanced in polish phase with full error handling and logging.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use super::session::SessionManager;
use crate::server::BinanceServer;
use crate::tools::chatgpt::{search_symbols, fetch_symbol_details};
use crate::binance::BinanceClient;

/// Shared state for SSE handlers
#[derive(Clone)]
pub struct SseState {
    pub session_manager: SessionManager,
    pub mcp_server: Arc<BinanceServer>,
    pub binance_client: Arc<BinanceClient>,
}

impl SseState {
    pub fn new(session_manager: SessionManager, mcp_server: BinanceServer) -> Self {
        Self {
            session_manager,
            mcp_server: Arc::new(mcp_server),
            binance_client: Arc::new(BinanceClient::new()),
        }
    }
}

/// Message POST - validates connection, routes to MCP server
///
/// Streamable HTTP transport (March 2025 spec):
/// - First request (initialize) creates session, returns Mcp-Session-Id header
/// - Subsequent requests must include Mcp-Session-Id header
/// - Returns JSON-RPC response as application/json (default)
/// - Can return text/event-stream for long-running operations (future)
pub async fn message_post(
    State(state): State<SseState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    // Extract method to check if this is an initialize request
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let is_initialize = method == "initialize";

    // Check for Mcp-Session-Id header (Streamable HTTP spec)
    let session_id = headers.get("Mcp-Session-Id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let connection_id = if is_initialize {
        // Initialize: Create new session (even if Mcp-Session-Id present)
        let addr = "127.0.0.1:0".parse().unwrap();
        match state.session_manager.register_connection(addr, None).await {
            Some(id) => {
                tracing::info!(session_id = %id, "New MCP session created (Streamable HTTP)");
                id
            }
            None => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": payload.get("id"),
                        "error": {
                            "code": -32000,
                            "message": "Maximum concurrent sessions reached (50)"
                        }
                    })),
                )
                    .into_response();
            }
        }
    } else {
        // Non-initialize: Require Mcp-Session-Id
        match session_id.as_ref() {
            Some(id) => {
                // Validate session exists
                if state.session_manager.get_session(id).await.is_none() {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": payload.get("id"),
                            "error": {
                                "code": -32001,
                                "message": "Session not found or expired"
                            }
                        })),
                    )
                        .into_response();
                }
                // Update activity
                state.session_manager.update_activity(id).await;
                id.clone()
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": payload.get("id"),
                        "error": {
                            "code": -32002,
                            "message": "Missing Mcp-Session-Id header"
                        }
                    })),
                )
                    .into_response();
            }
        }
    };

    // For MVP: Process JSON-RPC request synchronously and return as SSE event
    // This is a simplified implementation - proper async SSE streaming in Phase 6

    // Extract method and params from JSON-RPC request
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = payload.get("params").cloned().unwrap_or(Value::Null);
    let request_id = payload.get("id").cloned().unwrap_or(Value::Null);

    tracing::debug!(
        connection_id = %connection_id,
        method = %method,
        "Processing MCP request"
    );

    // Route to appropriate MCP handler based on method
    let result = match method {
        "initialize" => {
            // MCP initialize handshake - return server capabilities
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "Binance MCP Server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })
        }
        "tools/list" => {
            // Return full list of available tools (ChatGPT-compatible + Binance)
            serde_json::json!({
                "tools": [
                    {
                        "name": "search",
                        "description": "Search for cryptocurrency trading pairs by keyword (e.g., BTC, ETH, USDT). Returns top matching symbols with current prices.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "Search query - cryptocurrency symbol or name (e.g., 'BTC', 'ethereum', 'USDT pairs')"
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "fetch",
                        "description": "Fetch detailed market data for a specific trading symbol. Returns comprehensive information including 24h stats, order book depth, and trading rules.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "string",
                                    "description": "Trading symbol (e.g., BTCUSDT, ETHBTC) - use search to find available symbols"
                                }
                            },
                            "required": ["id"]
                        }
                    },
                    {
                        "name": "get_ticker",
                        "description": "Get current ticker price for a symbol",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "symbol": {
                                    "type": "string",
                                    "description": "Trading pair symbol (e.g., BTCUSDT)"
                                }
                            },
                            "required": ["symbol"]
                        }
                    },
                    {
                        "name": "get_exchange_info",
                        "description": "Get exchange information and trading rules",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "get_klines",
                        "description": "Get candlestick/kline data",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "symbol": {
                                    "type": "string",
                                    "description": "Trading pair symbol"
                                },
                                "interval": {
                                    "type": "string",
                                    "description": "Kline interval (1m, 5m, 1h, 1d, etc.)"
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Number of klines to return (max 1000)"
                                }
                            },
                            "required": ["symbol", "interval"]
                        }
                    }
                ]
            })
        }
        "tools/call" => {
            // Extract tool name and arguments
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

            tracing::info!(
                connection_id = %connection_id,
                tool = %tool_name,
                "Calling MCP tool"
            );

            // Route to appropriate tool handler
            // MCP requires results in content array format
            match tool_name {
                "search" => {
                    // ChatGPT search tool - search trading symbols
                    let query = arguments.get("query")
                        .and_then(|q| q.as_str())
                        .unwrap_or("");

                    match search_symbols(&state.binance_client, query).await {
                        Ok(results) => {
                            // MCP format: wrap in content array with type "text"
                            let results_json = serde_json::json!({"results": results});
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&results_json).unwrap()
                                }]
                            })
                        }
                        Err(e) => {
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("{{\"error\": \"Search failed: {}\"}}", e)
                                }],
                                "isError": true
                            })
                        }
                    }
                }
                "fetch" => {
                    // ChatGPT fetch tool - get detailed symbol info
                    let symbol_id = arguments.get("id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("");

                    match fetch_symbol_details(&state.binance_client, symbol_id).await {
                        Ok(details) => {
                            // MCP format: wrap in content array with type "text"
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&details).unwrap()
                                }]
                            })
                        }
                        Err(e) => {
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("{{\"error\": \"Fetch failed: {}\"}}", e)
                                }],
                                "isError": true
                            })
                        }
                    }
                }
                "get_ticker" => {
                    // Get ticker from Binance API
                    let symbol = arguments.get("symbol")
                        .and_then(|s| s.as_str())
                        .unwrap_or("BTCUSDT");

                    match state.binance_client.get_24hr_ticker(symbol).await {
                        Ok(ticker) => {
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&ticker).unwrap()
                                }]
                            })
                        }
                        Err(e) => {
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("{{\"error\": \"Failed to get ticker: {}\"}}", e)
                                }],
                                "isError": true
                            })
                        }
                    }
                }
                "get_klines" => {
                    // Get klines from Binance API
                    let symbol = arguments.get("symbol")
                        .and_then(|s| s.as_str())
                        .unwrap_or("BTCUSDT");
                    let interval = arguments.get("interval")
                        .and_then(|i| i.as_str())
                        .unwrap_or("1d");
                    let limit = arguments.get("limit")
                        .and_then(|l| l.as_u64())
                        .map(|l| l as u32);

                    match state.binance_client.get_klines(symbol, interval, limit).await {
                        Ok(klines) => {
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&klines).unwrap()
                                }]
                            })
                        }
                        Err(e) => {
                            serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("{{\"error\": \"Failed to get klines: {}\"}}", e)
                                }],
                                "isError": true
                            })
                        }
                    }
                }
                _ => {
                    serde_json::json!({
                        "error": format!("Unknown tool: {}", tool_name)
                    })
                }
            }
        }
        _ => {
            serde_json::json!({
                "error": format!("Unknown method: {}", method)
            })
        }
    };

    // Build JSON-RPC response
    let json_rpc_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": result
    });

    // Streamable HTTP transport (March 2025 spec):
    // Check Accept header to determine response format
    let accept = headers.get(axum::http::header::ACCEPT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/json");

    // Build response based on Accept header
    let mut response = if accept.contains("text/event-stream") {
        // Client wants SSE stream - return as SSE event
        let sse_event = format!("data: {}\n\n", serde_json::to_string(&json_rpc_response).unwrap());
        (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
            sse_event,
        )
            .into_response()
    } else {
        // Client wants JSON (default) - return plain JSON-RPC response
        (
            StatusCode::OK,
            Json(json_rpc_response),
        )
            .into_response()
    };

    // For initialize requests, add Mcp-Session-Id header (Streamable HTTP spec)
    if is_initialize {
        response.headers_mut().insert(
            "Mcp-Session-Id",
            connection_id.parse().unwrap(),
        );
        tracing::info!(session_id = %connection_id, "Returned Mcp-Session-Id in initialize response");
    }

    response
}

/// Root endpoint for MCP server discovery
///
/// Returns metadata about the MCP server for client discovery
pub async fn server_info() -> impl IntoResponse {
    let info = json!({
        "name": "Binance MCP Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Model Context Protocol server for Binance cryptocurrency exchange API",
        "protocol": "mcp",
        "transport": "streamable-http",
        "endpoints": {
            "mcp": "/mcp",
            "messages": "/messages",
            "tools": "/tools/list",
            "health": "/health"
        },
        "capabilities": {
            "tools": true,
            "prompts": false,
            "resources": false
        }
    });

    (StatusCode::OK, Json(info)).into_response()
}

// Streamable HTTP handlers are just aliases to existing handlers
// The router will handle GET vs POST routing

/// Tools list endpoint for OpenAI/ChatGPT MCP integration
///
/// Returns JSON-RPC response with list of available MCP tools
pub async fn tools_list(
    State(_state): State<SseState>,
) -> impl IntoResponse {
    // Return full list of tools including ChatGPT-required search/fetch
    let tools = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": [
                {
                    "name": "search",
                    "description": "Search for cryptocurrency trading pairs by keyword (e.g., BTC, ETH, USDT). Returns top matching symbols with current prices.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query - cryptocurrency symbol or name (e.g., 'BTC', 'ethereum', 'USDT pairs')"
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "fetch",
                    "description": "Fetch detailed market data for a specific trading symbol. Returns comprehensive information including 24h stats, order book depth, and trading rules.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Trading symbol (e.g., BTCUSDT, ETHBTC) - use search to find available symbols"
                            }
                        },
                        "required": ["id"]
                    }
                },
                {
                    "name": "get_ticker",
                    "description": "Get current ticker price for a symbol",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "symbol": {
                                "type": "string",
                                "description": "Trading pair symbol (e.g., BTCUSDT)"
                            }
                        },
                        "required": ["symbol"]
                    }
                },
                {
                    "name": "get_exchange_info",
                    "description": "Get exchange information and trading rules"
                },
                {
                    "name": "get_klines",
                    "description": "Get candlestick/kline data",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "symbol": {
                                "type": "string",
                                "description": "Trading pair symbol"
                            },
                            "interval": {
                                "type": "string",
                                "description": "Kline interval (1m, 5m, 1h, 1d, etc.)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Number of klines to return (max 1000)"
                            }
                        },
                        "required": ["symbol", "interval"]
                    }
                }
            ]
        }
    });

    (StatusCode::OK, Json(tools)).into_response()
}
