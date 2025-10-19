//! SSE endpoint handlers for remote MCP access
//!
//! Implements HTTP endpoints for SSE transport protocol:
//! - GET /mcp/sse: SSE handshake (returns connection ID)
//! - POST /mcp/message: JSON-RPC message exchange
//!
//! ## Implementation Tasks
//!
//! - T020: SSE handshake handler (GET /mcp/sse)
//! - T021: Message POST handler (POST /mcp/message)
//! - T024: Error handling for invalid connection-id (HTTP 404)
//! - T025: Max connections limit enforcement (HTTP 503)
//! - T026: Connection metadata logging (client IP, user-agent)

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, Sse},
    Json,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use super::session::SessionManager;
use crate::server::BinanceServer;

/// Shared state for SSE handlers
///
/// Contains session manager and MCP server instance for routing tool calls.
#[derive(Clone)]
pub struct SseState {
    /// Session manager for connection tracking
    pub session_manager: SessionManager,

    /// MCP server instance for handling tool calls
    pub mcp_server: Arc<BinanceServer>,
}

impl SseState {
    /// Creates new SSE state with session manager and MCP server
    pub fn new(session_manager: SessionManager, mcp_server: BinanceServer) -> Self {
        Self {
            session_manager,
            mcp_server: Arc::new(mcp_server),
        }
    }
}

/// T020: SSE handshake endpoint handler
///
/// Establishes new SSE connection and returns unique connection ID.
///
/// ## Request
///
/// ```http
/// GET /mcp/sse HTTP/1.1
/// Accept: text/event-stream
/// ```
///
/// ## Response (Success)
///
/// ```http
/// HTTP/1.1 200 OK
/// Content-Type: text/event-stream
/// X-Connection-ID: 550e8400-e29b-41d4-a716-446655440000
/// Cache-Control: no-cache
/// Connection: keep-alive
///
/// : connected
///
/// ```
///
/// ## Response (Max Connections Reached - T025)
///
/// ```http
/// HTTP/1.1 503 Service Unavailable
/// Content-Type: application/json
///
/// {"error": "Maximum concurrent connections reached (50)"}
/// ```
///
/// ## Connection Metadata Logging (T026)
///
/// Logs at DEBUG level:
/// - Client IP address
/// - User-Agent header (if present)
/// - Connection ID
/// - Total active connections
pub async fn sse_handshake(
    State(state): State<SseState>,
    headers: HeaderMap,
) -> Response {
    // TODO: Extract client address from ConnectInfo in production (T026 polish)
    // For MVP, use placeholder address
    let addr = "127.0.0.1:0".parse().unwrap();
    // Extract User-Agent header for logging (T026)
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // T026: Log connection attempt with metadata
    tracing::debug!(
        client_addr = %addr,
        user_agent = ?user_agent,
        "SSE connection attempt"
    );

    // T025: Register connection (enforces max 50 limit)
    let connection_id = match state
        .session_manager
        .register_connection(addr, user_agent.clone())
        .await
    {
        Some(id) => id,
        None => {
            // Max connections reached - return HTTP 503
            tracing::warn!(
                client_addr = %addr,
                "Rejected SSE connection: max concurrent connections reached"
            );

            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Maximum concurrent connections reached (50)"
                })),
            )
                .into_response();
        }
    };

    // T026: Log successful connection with metadata
    tracing::info!(
        connection_id = %connection_id,
        client_addr = %addr,
        user_agent = ?user_agent,
        active_connections = state.session_manager.connection_count().await,
        "SSE connection established"
    );

    // Create SSE event stream
    let (tx, rx) = mpsc::channel::<String>(32);

    // Send initial connected event
    let _ = tx.send(": connected\n\n".to_string()).await;

    // Store event sender in session for future message delivery
    // TODO: Store tx in SessionMetadata for message routing (T021)

    // Convert receiver to SSE stream
    let stream = ReceiverStream::new(rx).map(|data| Ok::<_, axum::Error>(axum::response::sse::Event::default().data(data)));

    // Build SSE response with connection ID header
    let mut response = Sse::new(stream).into_response();

    // Add X-Connection-ID header
    response.headers_mut().insert(
        "X-Connection-ID",
        connection_id.parse().unwrap(),
    );

    response
}

/// T021: Message POST endpoint handler
///
/// Receives JSON-RPC messages and routes to MCP server.
///
/// ## Request
///
/// ```http
/// POST /mcp/message HTTP/1.1
/// Content-Type: application/json
/// X-Connection-ID: 550e8400-e29b-41d4-a716-446655440000
///
/// {
///   "jsonrpc": "2.0",
///   "id": 1,
///   "method": "tools/call",
///   "params": {
///     "name": "get_ticker",
///     "arguments": {"symbol": "BTCUSDT"}
///   }
/// }
/// ```
///
/// ## Response (Success)
///
/// ```http
/// HTTP/1.1 202 Accepted
/// Content-Type: application/json
///
/// {"status": "processing"}
/// ```
///
/// ## Response (Invalid Connection ID - T024)
///
/// ```http
/// HTTP/1.1 404 Not Found
/// Content-Type: application/json
///
/// {"error": "Connection not found"}
/// ```
///
/// ## Response (Missing Connection ID)
///
/// ```http
/// HTTP/1.1 400 Bad Request
/// Content-Type: application/json
///
/// {"error": "Missing X-Connection-ID header"}
/// ```
pub async fn message_post(
    State(_state): State<SseState>,
    headers: HeaderMap,
    Json(_payload): Json<Value>,
) -> Response {
    // Validate X-Connection-ID header exists
    let connection_id = match headers.get("X-Connection-ID") {
        Some(header_value) => match header_value.to_str() {
            Ok(id) => id,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Invalid X-Connection-ID header"
                    })),
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Missing X-Connection-ID header"
                })),
            )
                .into_response();
        }
    };

    // T024: Validate connection exists
    let session_exists = _state
        .session_manager
        .get_session(connection_id)
        .await
        .is_some();

    if !session_exists {
        tracing::warn!(
            connection_id = %connection_id,
            "Message received for non-existent connection"
        );

        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Connection not found"
            })),
        )
            .into_response();
    }

    // TODO: Route JSON-RPC message to MCP server (T021, T023)
    // TODO: Send response via SSE event stream (T022)

    // Return 202 Accepted immediately
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "processing"
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_state_creation() {
        let session_manager = SessionManager::new();
        let mcp_server = BinanceServer::new();
        let state = SseState::new(session_manager, mcp_server);

        // Verify state is created successfully
        assert_eq!(
            Arc::strong_count(&state.mcp_server),
            1,
            "MCP server should have single Arc reference"
        );
    }
}
