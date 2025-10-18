//! @aggTrade WebSocket stream for volume profile analysis
//!
//! Connects to Binance aggregate trade stream (wss://stream.binance.com:9443/ws/<symbol>@aggTrade)
//! with exponential backoff reconnection (1s, 2s, 4s, 8s, max 60s).

use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Aggregate trade event from Binance @aggTrade stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggTrade {
    /// Event type (always "aggTrade")
    #[serde(rename = "e")]
    pub event_type: String,

    /// Event time (milliseconds)
    #[serde(rename = "E")]
    pub event_time: u64,

    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,

    /// Aggregate trade ID
    #[serde(rename = "a")]
    pub agg_trade_id: u64,

    /// Price (string to preserve precision)
    #[serde(rename = "p")]
    pub price: String,

    /// Quantity (string to preserve precision)
    #[serde(rename = "q")]
    pub quantity: String,

    /// First trade ID
    #[serde(rename = "f")]
    pub first_trade_id: u64,

    /// Last trade ID
    #[serde(rename = "l")]
    pub last_trade_id: u64,

    /// Trade time (milliseconds)
    #[serde(rename = "T")]
    pub trade_time: u64,

    /// Is buyer the market maker? (true = sell, false = buy)
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

/// Connect to Binance @aggTrade WebSocket stream with exponential backoff (T026-T027)
///
/// Implements reconnection logic:
/// - Initial delay: 1 second
/// - Exponential backoff: 2x each retry (2s, 4s, 8s, 16s...)
/// - Maximum delay: 60 seconds
///
/// # Example
/// ```no_run
/// # use mcp_binance_server::orderbook::analytics::trade_stream::*;
/// # async fn example() -> anyhow::Result<()> {
/// let (mut trade_rx, handle) = connect_trade_stream("btcusdt").await?;
///
/// while let Some(trade) = trade_rx.recv().await {
///     println!("Trade: {} @ {}", trade.quantity, trade.price);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn connect_trade_stream(
    symbol: &str,
) -> Result<(
    tokio::sync::mpsc::Receiver<AggTrade>,
    tokio::task::JoinHandle<()>,
)> {
    let symbol_lower = symbol.to_lowercase();
    let url = format!("wss://stream.binance.com:9443/ws/{}@aggTrade", symbol_lower);

    let (tx, rx) = tokio::sync::mpsc::channel(1000);

    let handle = tokio::spawn(async move {
        let mut retry_delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);

        loop {
            match connect_and_stream(&url, tx.clone()).await {
                Ok(_) => {
                    tracing::info!("@aggTrade stream disconnected gracefully");
                    retry_delay = Duration::from_secs(1); // Reset on clean disconnect
                }
                Err(e) => {
                    tracing::error!(
                        "@aggTrade stream error: {}, retrying in {:?}",
                        e,
                        retry_delay
                    );
                }
            }

            sleep(retry_delay).await;

            // Exponential backoff with max cap
            retry_delay = std::cmp::min(retry_delay * 2, max_delay);
        }
    });

    Ok((rx, handle))
}

/// Internal: Connect and stream trades until error or disconnect
async fn connect_and_stream(url: &str, tx: tokio::sync::mpsc::Sender<AggTrade>) -> Result<()> {
    let (ws_stream, _) = connect_async(url)
        .await
        .context("Failed to connect to @aggTrade WebSocket")?;

    tracing::info!("Connected to @aggTrade stream: {}", url);

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg.context("WebSocket message error")?;

        if let Message::Text(text) = msg {
            match serde_json::from_str::<AggTrade>(&text) {
                Ok(trade) => {
                    if tx.send(trade).await.is_err() {
                        tracing::warn!("Trade receiver dropped, closing stream");
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse @aggTrade event: {}", e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agg_trade_deserialization() {
        let json = r#"{
            "e": "aggTrade",
            "E": 1737158400000,
            "s": "BTCUSDT",
            "a": 12345,
            "p": "50000.50",
            "q": "0.5",
            "f": 100,
            "l": 105,
            "T": 1737158400000,
            "m": false
        }"#;

        let trade: AggTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.price, "50000.50");
        assert_eq!(trade.quantity, "0.5");
        assert!(!trade.is_buyer_maker);
    }
}
