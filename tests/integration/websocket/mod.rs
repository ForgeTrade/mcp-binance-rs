//! WebSocket integration tests
//!
//! Comprehensive test suite for 3 WebSocket streams:
//! - Ticker stream: 24hr ticker updates for symbols
//! - Depth stream: Order book depth updates
//! - User data stream: Account updates, order updates, trade updates

pub mod streams;

use crate::common::fixtures::TestCredentials;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Helper: Connect to WebSocket stream
pub async fn connect_websocket(
    url: &str,
) -> Result<
    (
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    ),
    tokio_tungstenite::tungstenite::Error,
> {
    connect_async(url).await
}

/// Helper: Wait for WebSocket message with timeout
pub async fn receive_message_with_timeout(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    timeout_secs: u64,
) -> Result<Option<Message>, String> {
    match timeout(Duration::from_secs(timeout_secs), ws_stream.next()).await {
        Ok(Some(Ok(msg))) => Ok(Some(msg)),
        Ok(Some(Err(e))) => Err(format!("WebSocket error: {}", e)),
        Ok(None) => Ok(None), // Stream closed
        Err(_) => Err("Timeout waiting for message".to_string()),
    }
}

/// Helper: Parse JSON message from WebSocket
pub fn parse_json_message(msg: &Message) -> Result<serde_json::Value, String> {
    match msg {
        Message::Text(text) => {
            serde_json::from_str(text).map_err(|e| format!("JSON parse error: {}", e))
        }
        _ => Err(format!("Expected text message, got: {:?}", msg)),
    }
}

/// Helper: Send ping to WebSocket to keep connection alive
pub async fn send_ping(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Result<(), String> {
    ws_stream
        .send(Message::Ping(vec![].into()))
        .await
        .map_err(|e| format!("Failed to send ping: {}", e))
}

/// Helper: Close WebSocket connection gracefully
pub async fn close_websocket(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Result<(), String> {
    ws_stream
        .close(None)
        .await
        .map_err(|e| format!("Failed to close WebSocket: {}", e))
}

/// Helper: Build WebSocket URL for stream
pub fn build_stream_url(base_ws_url: &str, stream_name: &str) -> String {
    format!("{}/ws/{}", base_ws_url, stream_name)
}

/// Helper: Get test credentials with WebSocket URL
pub fn get_test_ws_config() -> (String, TestCredentials) {
    let creds = TestCredentials::from_env();
    let ws_url = std::env::var("BINANCE_TESTNET_WS_URL")
        .unwrap_or_else(|_| "wss://stream.testnet.binance.vision".to_string());
    (ws_url, creds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_stream_url() {
        let base = "wss://stream.testnet.binance.vision";
        let stream = "btcusdt@ticker";
        let url = build_stream_url(base, stream);

        assert_eq!(url, "wss://stream.testnet.binance.vision/ws/btcusdt@ticker");
    }

    #[test]
    fn test_get_test_ws_config() {
        let (ws_url, creds) = get_test_ws_config();

        assert!(!ws_url.is_empty());
        assert!(ws_url.starts_with("wss://"));
        assert!(!creds.api_key.is_empty());
    }

    #[test]
    fn test_parse_json_message() {
        let json_text = r#"{"e":"ticker","s":"BTCUSDT","p":"100.00"}"#;
        let msg = Message::Text(json_text.to_string().into());

        let result = parse_json_message(&msg);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["e"], "ticker");
        assert_eq!(json["s"], "BTCUSDT");
    }
}
