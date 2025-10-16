//! User Data WebSocket Handler
//!
//! Provides WebSocket endpoint for real-time order and balance updates.
//! Clients connect to `/ws/user` and receive JSON user data events.
//!
//! ## Features
//! - Real-time order execution reports (fills, cancellations)
//! - Real-time balance updates from trades
//! - Automatic listen key creation and renewal
//! - Client connection management and cleanup
//! - Authentication via Bearer token in upgrade request

#[cfg(feature = "http-api")]
use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};

#[cfg(all(feature = "http-api", feature = "websocket"))]
use crate::binance::websocket::{BinanceWebSocketClient, UserDataEvent};
#[cfg(feature = "http-api")]
use crate::http::AppState;
#[cfg(all(feature = "http-api", feature = "websocket"))]
use axum::extract::ws::{Message, WebSocket};
#[cfg(all(feature = "http-api", feature = "websocket"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(all(feature = "http-api", feature = "websocket"))]
use std::time::Duration;
#[cfg(all(feature = "http-api", feature = "websocket"))]
use tokio::sync::broadcast;

/// Listen key renewal interval (30 minutes)
/// Binance listen keys expire after 60 minutes, so we renew at 30 minutes
#[cfg(all(feature = "http-api", feature = "websocket"))]
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// WebSocket upgrade handler for user data stream
///
/// Upgrades HTTP connection to WebSocket and starts forwarding
/// user data events (order updates, balance changes) from Binance to the client.
///
/// ## Endpoint
/// `GET /ws/user`
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
/// wscat -c 'ws://localhost:3000/ws/user' \
///   -H "Authorization: Bearer test_token"
/// ```
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub async fn user_data_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    tracing::info!("WebSocket upgrade request for user data");

    // Try to acquire connection permit (non-blocking)
    let permit = match state.ws_connections.clone().try_acquire_owned() {
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

    ws.on_upgrade(move |socket| handle_user_data_socket(socket, state, permit))
}

/// Handle individual user data WebSocket connection
///
/// Creates listen key, subscribes to Binance user data broadcast channel,
/// forwards messages to client WebSocket, and manages listen key renewal.
///
/// ## Arguments
/// - `socket`: WebSocket connection to the client
/// - `state`: Shared application state with Binance client
/// - `_permit`: Connection permit from semaphore (held until socket closes)
#[cfg(all(feature = "http-api", feature = "websocket"))]
async fn handle_user_data_socket(
    socket: WebSocket,
    state: AppState,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    tracing::info!("User data WebSocket connected (permit acquired)");

    // Create listen key
    let listen_key = match state.binance_client.create_listen_key().await {
        Ok(key) => {
            tracing::info!("Created listen key for user data stream");
            key
        }
        Err(e) => {
            tracing::error!("Failed to create listen key: {}", e);
            return;
        }
    };

    // Create broadcast channel for this user
    // Channel size of 100 messages to handle bursts
    let (tx, mut rx) = broadcast::channel::<UserDataEvent>(100);

    // Start Binance stream task
    let ws_client = BinanceWebSocketClient::new();
    let listen_key_clone = listen_key.clone();
    let binance_task = tokio::spawn(async move {
        if let Err(e) = ws_client.user_data_stream_task(&listen_key_clone, tx).await {
            tracing::error!("User data stream task failed: {}", e);
        }
    });

    // Start keepalive task
    let binance_client = state.binance_client.clone();
    let listen_key_clone = listen_key.clone();
    let keepalive_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(KEEPALIVE_INTERVAL).await;

            match binance_client.keepalive_listen_key(&listen_key_clone).await {
                Ok(_) => {
                    tracing::info!("Listen key renewed successfully");
                }
                Err(e) => {
                    tracing::error!("Failed to renew listen key: {}", e);
                    break;
                }
            }
        }
    });

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward broadcast messages to client
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Serialize user data event to JSON
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            // Send to client
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                tracing::info!("Client disconnected");
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to serialize user data event: {}", e);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    // Client is falling behind - log warning for T068
                    tracing::warn!("User data stream lagging: {} messages skipped", skipped);
                    // Continue receiving after lag
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("User data broadcast channel closed");
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
            tracing::info!("Send task completed");
            recv_task.abort();
        },
        _ = &mut recv_task => {
            tracing::info!("Receive task completed");
            send_task.abort();
        },
    }

    // Clean up tasks
    binance_task.abort();
    keepalive_task.abort();

    // Close listen key
    if let Err(e) = state.binance_client.close_listen_key(&listen_key).await {
        tracing::warn!("Failed to close listen key: {}", e);
    } else {
        tracing::info!("Listen key closed successfully");
    }

    tracing::info!("User data WebSocket disconnected (permit released)");
    // Permit is automatically released when _permit is dropped
}
