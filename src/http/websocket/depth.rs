//! Depth (Order Book) WebSocket Handler
//!
//! Provides WebSocket endpoint for real-time order book depth updates.
//! Clients connect to `/ws/depth/:symbol` and receive JSON depth messages.
//!
//! ## Features
//! - Real-time bid/ask price level updates
//! - Automatic subscription to Binance depth stream
//! - Client connection management and cleanup
//! - Authentication via Bearer token in upgrade request

#[cfg(feature = "http-api")]
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};

#[cfg(all(feature = "http-api", feature = "websocket"))]
use crate::binance::websocket::{BinanceWebSocketClient, DepthUpdate};
#[cfg(feature = "http-api")]
use crate::http::AppState;
#[cfg(all(feature = "http-api", feature = "websocket"))]
use axum::extract::ws::{Message, WebSocket};
#[cfg(all(feature = "http-api", feature = "websocket"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(all(feature = "http-api", feature = "websocket"))]
use tokio::sync::broadcast;

/// WebSocket upgrade handler for depth stream
///
/// Upgrades HTTP connection to WebSocket and starts forwarding
/// order book depth updates from Binance to the client.
///
/// ## Endpoint
/// `GET /ws/depth/:symbol`
///
/// ## Authentication
/// Requires valid Bearer token in Authorization header
///
/// ## Connection Limit
/// Maximum 50 concurrent WebSocket connections (SC-003 requirement).
/// Returns HTTP 503 if limit exceeded.
///
/// ## Example
/// ```bash
/// wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
///   -H "Authorization: Bearer test_token"
/// ```
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub async fn depth_handler(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    tracing::info!("WebSocket upgrade request for depth: {}", symbol);

    // Try to acquire connection permit (non-blocking)
    let permit = match state.ws_connections.try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            tracing::warn!("WebSocket connection limit reached (50 concurrent)");
            return axum::response::Response::builder()
                .status(503)
                .header("Retry-After", "30")
                .body("Service Unavailable: Maximum WebSocket connections reached".into())
                .unwrap();
        }
    };

    ws.on_upgrade(move |socket| handle_depth_socket(socket, symbol, permit))
}

/// Handle individual depth WebSocket connection
///
/// Creates subscription to Binance depth broadcast channel and
/// forwards messages to client WebSocket.
///
/// ## Arguments
/// - `socket`: WebSocket connection to the client
/// - `symbol`: Trading pair symbol (e.g., "btcusdt")
/// - `_permit`: Connection permit from semaphore (held until socket closes)
#[cfg(all(feature = "http-api", feature = "websocket"))]
async fn handle_depth_socket(
    socket: WebSocket,
    symbol: String,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    tracing::info!("Depth WebSocket connected for {} (permit acquired)", symbol);

    // Create broadcast channel for this symbol
    // Channel size of 100 messages to handle bursts
    let (tx, mut rx) = broadcast::channel::<DepthUpdate>(100);

    // Start Binance stream task
    let ws_client = BinanceWebSocketClient::new();
    let symbol_clone = symbol.clone();
    tokio::spawn(async move {
        if let Err(e) = ws_client.depth_stream_task(&symbol_clone, tx).await {
            tracing::error!("Depth stream task failed: {}", e);
        }
    });

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward broadcast messages to client
    let symbol_for_task = symbol.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(update) => {
                    // Serialize depth update to JSON
                    match serde_json::to_string(&update) {
                        Ok(json) => {
                            // Send to client
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                tracing::info!("Client disconnected");
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to serialize depth update: {}", e);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    // Client is falling behind - log warning for T068
                    tracing::warn!(
                        "Depth stream lagging for {}: {} messages skipped",
                        symbol_for_task,
                        skipped
                    );
                    // Continue receiving after lag
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Depth broadcast channel closed");
                    break;
                }
            }
        }
    });

    // Spawn task to handle client messages (pings, close frames)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    tracing::info!("Client sent close frame");
                    break;
                }
                Message::Ping(data) => {
                    tracing::debug!("Received ping from client");
                    // Pong is handled automatically by axum
                    drop(data);
                }
                _ => {
                    tracing::debug!("Received message from client: {:?}", msg);
                }
            }
        }
    });

    // Wait for either task to complete (disconnect or error)
    tokio::select! {
        _ = &mut send_task => {
            tracing::info!("Send task completed for {}", symbol);
            recv_task.abort();
        },
        _ = &mut recv_task => {
            tracing::info!("Receive task completed for {}", symbol);
            send_task.abort();
        },
    }

    tracing::info!(
        "Depth WebSocket disconnected for {} (permit released)",
        symbol
    );
    // Permit is automatically released when _permit is dropped
}
