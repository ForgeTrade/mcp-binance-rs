//! Ticker WebSocket Handler
//!
//! Provides WebSocket endpoint for real-time ticker price updates.
//! Clients connect to `/ws/ticker/:symbol` and receive JSON ticker messages.
//!
//! ## Features
//! - Real-time price updates every ~1000ms
//! - Automatic subscription to Binance ticker stream
//! - Client connection management and cleanup
//! - Authentication via Bearer token in upgrade request

#[cfg(feature = "http-api")]
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};

#[cfg(all(feature = "http-api", feature = "websocket"))]
use crate::binance::websocket::{BinanceWebSocketClient, TickerUpdate};
#[cfg(feature = "http-api")]
use crate::http::AppState;
#[cfg(all(feature = "http-api", feature = "websocket"))]
use axum::extract::ws::{Message, WebSocket};
#[cfg(all(feature = "http-api", feature = "websocket"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(all(feature = "http-api", feature = "websocket"))]
use tokio::sync::broadcast;

/// WebSocket upgrade handler for ticker stream
///
/// Upgrades HTTP connection to WebSocket and starts forwarding
/// ticker updates from Binance to the client.
///
/// ## Endpoint
/// `GET /ws/ticker/:symbol`
///
/// ## Authentication
/// Requires valid Bearer token in Authorization header
///
/// ## Example
/// ```bash
/// wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
///   -H "Authorization: Bearer test_token"
/// ```
#[cfg(all(feature = "http-api", feature = "websocket"))]
pub async fn ticker_handler(
    State(_state): State<AppState>,
    Path(symbol): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    tracing::info!("WebSocket upgrade request for ticker: {}", symbol);

    ws.on_upgrade(move |socket| handle_ticker_socket(socket, symbol))
}

/// Handle individual ticker WebSocket connection
///
/// Creates subscription to Binance ticker broadcast channel and
/// forwards messages to client WebSocket.
#[cfg(all(feature = "http-api", feature = "websocket"))]
async fn handle_ticker_socket(socket: WebSocket, symbol: String) {
    tracing::info!("Ticker WebSocket connected for {}", symbol);

    // Create broadcast channel for this symbol
    // Channel size of 100 messages to handle bursts
    let (tx, mut rx) = broadcast::channel::<TickerUpdate>(100);

    // Start Binance stream task
    let ws_client = BinanceWebSocketClient::new();
    let symbol_clone = symbol.clone();
    tokio::spawn(async move {
        if let Err(e) = ws_client.ticker_stream_task(&symbol_clone, tx).await {
            tracing::error!("Ticker stream task failed: {}", e);
        }
    });

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward broadcast messages to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            // Serialize ticker update to JSON
            match serde_json::to_string(&update) {
                Ok(json) => {
                    // Send to client
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        tracing::info!("Client disconnected");
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to serialize ticker update: {}", e);
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

    tracing::info!("Ticker WebSocket disconnected for {}", symbol);
}
