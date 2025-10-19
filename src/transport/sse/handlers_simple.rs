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
use crate::server::tool_router::*; // Import all parameter types
use rmcp::handler::server::wrapper::Parameters;

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
            // Get tools from rmcp SDK router
            let sdk_tools = state.mcp_server.tool_router.list_all();

            // Add ChatGPT-required tools (search, fetch)
            let mut all_tools: Vec<serde_json::Value> = sdk_tools
                .iter()
                .map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema
                    })
                })
                .collect();

            // Prepend ChatGPT tools (search, fetch)
            all_tools.insert(0, serde_json::json!({
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
            }));
            all_tools.insert(1, serde_json::json!({
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
            }));

            serde_json::json!({
                "tools": all_tools
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
                // SDK tools - call methods directly with deserialized parameters
                "get_server_time" => {
                    match state.mcp_server.get_server_time().await {
                        Ok(result) => serde_json::to_value(&result).unwrap(),
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_ticker" => {
                    match serde_json::from_value::<SymbolParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_ticker(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_klines" => {
                    match serde_json::from_value::<KlinesParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_klines(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_order_book" => {
                    match serde_json::from_value::<OrderBookParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_order_book(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_recent_trades" => {
                    match serde_json::from_value::<RecentTradesParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_recent_trades(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_average_price" => {
                    match serde_json::from_value::<SymbolParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_average_price(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_account_info" => {
                    match state.mcp_server.get_account_info().await {
                        Ok(result) => serde_json::to_value(&result).unwrap(),
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_account_trades" => {
                    match serde_json::from_value::<AccountTradesParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_account_trades(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "place_order" => {
                    match serde_json::from_value::<PlaceOrderParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.place_order(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_order" => {
                    match serde_json::from_value::<OrderParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_order(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "cancel_order" => {
                    match serde_json::from_value::<OrderParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.cancel_order(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_open_orders" => {
                    match serde_json::from_value::<OpenOrdersParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_open_orders(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                "get_all_orders" => {
                    match serde_json::from_value::<AllOrdersParam>(arguments.clone()) {
                        Ok(params) => match state.mcp_server.get_all_orders(Parameters(params)).await {
                            Ok(result) => serde_json::to_value(&result).unwrap(),
                            Err(e) => serde_json::json!({
                                "content": [{"type": "text", "text": format!("{{\"error\": \"{}\"}}", e)}],
                                "isError": true
                            })
                        },
                        Err(e) => serde_json::json!({
                            "content": [{"type": "text", "text": format!("{{\"error\": \"Invalid parameters: {}\"}}", e)}],
                            "isError": true
                        })
                    }
                }
                _ => {
                    serde_json::json!({
                        "content": [{"type": "text", "text": format!("{{\"error\": \"Unknown tool: {}\"}}", tool_name)}],
                        "isError": true
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
    State(state): State<SseState>,
) -> impl IntoResponse {
    // Get tools from rmcp SDK router
    let sdk_tools = state.mcp_server.tool_router.list_all();

    // Add ChatGPT-required tools (search, fetch)
    let mut all_tools: Vec<serde_json::Value> = sdk_tools
        .iter()
        .map(|tool| {
            serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema
            })
        })
        .collect();

    // Prepend ChatGPT tools (search, fetch)
    all_tools.insert(0, serde_json::json!({
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
    }));
    all_tools.insert(1, serde_json::json!({
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
    }));

    let tools = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": all_tools
        }
    });

    (StatusCode::OK, Json(tools)).into_response()
}
