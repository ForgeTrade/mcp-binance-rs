//! Binance WebSocket Client
//!
//! Connects to Binance WebSocket streams for real-time market data.
//! Handles automatic reconnection with exponential backoff and message broadcasting.
//!
//! ## Features
//! - Ticker price streams (real-time price updates)
//! - Order book depth streams (bid/ask updates)
//! - User data streams (order/balance notifications)
//! - Automatic reconnection with exponential backoff (100ms → 30s)
//! - Message broadcasting via tokio::sync::broadcast channels

use crate::error::McpError;
use futures_util::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Base URL for Binance WebSocket streams
const BINANCE_WS_URL: &str = "wss://stream.binance.com:9443/ws";

/// Maximum reconnection backoff duration
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Initial reconnection backoff duration
const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

/// Binance WebSocket client for managing stream connections
///
/// Handles connections to Binance WebSocket API with automatic
/// reconnection and message broadcasting to multiple subscribers.
#[derive(Debug, Clone)]
pub struct BinanceWebSocketClient {
    /// Base WebSocket URL
    pub base_url: String,
}

impl BinanceWebSocketClient {
    /// Create a new Binance WebSocket client with default URL
    pub fn new() -> Self {
        Self {
            base_url: BINANCE_WS_URL.to_string(),
        }
    }

    /// Connect to a WebSocket stream with automatic retry and exponential backoff
    ///
    /// Retries connection failures with exponential backoff starting at 100ms
    /// and capping at 30 seconds between attempts.
    ///
    /// ## Arguments
    /// - `stream_name`: The Binance stream endpoint (e.g., "btcusdt@ticker", "btcusdt@depth")
    ///
    /// ## Returns
    /// WebSocket connection (write, read) split tuple
    ///
    /// ## Example
    /// ```rust,no_run
    /// use mcp_binance_server::binance::websocket::BinanceWebSocketClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BinanceWebSocketClient::new();
    /// let (_write, _read) = client.connect_with_retry("btcusdt@ticker").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_retry(
        &self,
        stream_name: &str,
    ) -> Result<
        (
            futures_util::stream::SplitSink<
                tokio_tungstenite::WebSocketStream<
                    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                >,
                Message,
            >,
            futures_util::stream::SplitStream<
                tokio_tungstenite::WebSocketStream<
                    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                >,
            >,
        ),
        McpError,
    > {
        let url = format!("{}/{}", self.base_url, stream_name);
        let mut backoff = INITIAL_BACKOFF;

        loop {
            tracing::info!("Connecting to Binance WebSocket: {}", url);

            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    tracing::info!("Connected to Binance WebSocket: {}", stream_name);
                    let (write, read) = ws_stream.split();
                    return Ok((write, read));
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to connect to {}: {}. Retrying in {:?}",
                        stream_name,
                        e,
                        backoff
                    );

                    sleep(backoff).await;

                    // Exponential backoff with cap
                    backoff = std::cmp::min(backoff * 2, MAX_BACKOFF);
                }
            }
        }
    }

    /// Start a ticker stream task that reads from Binance and broadcasts to subscribers
    ///
    /// Creates a background task that:
    /// 1. Connects to Binance ticker WebSocket stream
    /// 2. Reads ticker update messages
    /// 3. Broadcasts messages to all subscribers via broadcast channel
    /// 4. Automatically reconnects on connection loss
    ///
    /// ## Arguments
    /// - `symbol`: Trading pair symbol in lowercase (e.g., "btcusdt")
    /// - `tx`: Broadcast sender for distributing ticker updates to subscribers
    ///
    /// ## Returns
    /// Task handle that can be awaited or spawned
    ///
    /// ## Example
    /// ```rust,no_run
    /// use mcp_binance_server::binance::websocket::BinanceWebSocketClient;
    /// use tokio::sync::broadcast;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BinanceWebSocketClient::new();
    /// let (tx, _rx) = broadcast::channel(100);
    ///
    /// // Spawn task to run in background
    /// tokio::spawn(async move {
    ///     if let Err(e) = client.ticker_stream_task("btcusdt", tx).await {
    ///         eprintln!("Ticker stream error: {}", e);
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ticker_stream_task(
        &self,
        symbol: &str,
        tx: broadcast::Sender<TickerUpdate>,
    ) -> Result<(), McpError> {
        let stream_name = format!("{}@ticker", symbol.to_lowercase());

        loop {
            tracing::info!("Starting ticker stream for {}", symbol);

            // Connect with retry
            let (_write, mut read) = self.connect_with_retry(&stream_name).await?;

            // Read messages and broadcast to subscribers
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        // Parse ticker update
                        match serde_json::from_str::<TickerUpdate>(&text) {
                            Ok(update) => {
                                // Broadcast to all subscribers
                                // Ignore send errors (no active receivers)
                                let _ = tx.send(update);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse ticker update: {}", e);
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        tracing::debug!("Received ping with {} bytes", data.len());
                    }
                    Ok(Message::Pong(_)) => {
                        tracing::debug!("Received pong");
                    }
                    Ok(Message::Close(frame)) => {
                        tracing::info!("WebSocket closed: {:?}", frame);
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket read error: {}", e);
                        break;
                    }
                    _ => {
                        tracing::debug!("Received other message type");
                    }
                }
            }

            tracing::warn!("Ticker stream disconnected, reconnecting...");
            sleep(Duration::from_secs(1)).await;
        }
    }
}

impl Default for BinanceWebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Ticker price update message from Binance WebSocket
///
/// Received from the `<symbol>@ticker` stream every 1000ms
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerUpdate {
    /// Event type (always "24hrTicker")
    #[serde(rename = "e")]
    pub event_type: String,

    /// Event time (milliseconds since Unix epoch)
    #[serde(rename = "E")]
    pub event_time: i64,

    /// Trading pair symbol
    #[serde(rename = "s")]
    pub symbol: String,

    /// Price change
    #[serde(rename = "p")]
    pub price_change: String,

    /// Price change percent
    #[serde(rename = "P")]
    pub price_change_percent: String,

    /// Weighted average price
    #[serde(rename = "w")]
    pub weighted_avg_price: String,

    /// Last price
    #[serde(rename = "c")]
    pub last_price: String,

    /// Last quantity
    #[serde(rename = "Q")]
    pub last_quantity: String,

    /// Open price
    #[serde(rename = "o")]
    pub open_price: String,

    /// High price
    #[serde(rename = "h")]
    pub high_price: String,

    /// Low price
    #[serde(rename = "l")]
    pub low_price: String,

    /// Total traded base asset volume
    #[serde(rename = "v")]
    pub volume: String,

    /// Total traded quote asset volume
    #[serde(rename = "q")]
    pub quote_volume: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_ws_client_creation() {
        let client = BinanceWebSocketClient::new();
        assert_eq!(client.base_url, BINANCE_WS_URL);
    }

    #[test]
    fn test_ticker_update_deserialization() {
        let json = r#"{
            "e": "24hrTicker",
            "E": 123456789,
            "s": "BTCUSDT",
            "p": "100.00",
            "P": "0.50",
            "w": "45000.50",
            "c": "45100.00",
            "Q": "0.001",
            "o": "45000.00",
            "h": "45200.00",
            "l": "44900.00",
            "v": "1000.5",
            "q": "45000000.00"
        }"#;

        let update: TickerUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.symbol, "BTCUSDT");
        assert_eq!(update.last_price, "45100.00");
        assert_eq!(update.price_change, "100.00");
    }
}
