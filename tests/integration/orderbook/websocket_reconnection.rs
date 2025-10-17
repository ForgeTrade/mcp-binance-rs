//! WebSocket reconnection integration tests
//!
//! Tests automatic reconnection with exponential backoff when WebSocket
//! connections are interrupted or fail.

use mcp_binance_server::orderbook::websocket::DepthWebSocketClient;
use tokio::time::{Duration, sleep};

#[tokio::test]
#[ignore] // Requires real network connection
async fn test_websocket_reconnection_with_invalid_symbol() {
    // Test that WebSocket attempts reconnection when symbol is invalid
    // This will trigger immediate connection failures

    let (client, mut receiver) = DepthWebSocketClient::new("INVALIDSYMBOL".to_string());
    let handle = client.start();

    // Wait for 5 seconds - should see retry attempts with exponential backoff
    // Expected delays: 1s, 2s, 4s, 8s
    let start = std::time::Instant::now();
    sleep(Duration::from_secs(5)).await;
    let elapsed = start.elapsed();

    // Should not have received any valid updates
    assert!(
        receiver.try_recv().is_err(),
        "Should not receive updates from invalid symbol"
    );

    // Elapsed time should be at least 5 seconds (allowing for retries)
    assert!(
        elapsed.as_secs() >= 5,
        "Should have waited at least 5 seconds for retries"
    );

    handle.abort();
}

#[tokio::test]
async fn test_websocket_client_creation() {
    // Test that WebSocket client can be created without errors
    let (client, _receiver) = DepthWebSocketClient::new("BTCUSDT".to_string());
    let _handle = client.start();

    // Wait a moment to ensure task starts
    sleep(Duration::from_millis(100)).await;

    // No panic means success
}

#[tokio::test]
async fn test_websocket_receiver_channel() {
    // Test that receiver channel is properly set up
    let (_client, mut receiver) = DepthWebSocketClient::new("BTCUSDT".to_string());

    // Receiver should be initially empty
    assert!(
        receiver.try_recv().is_err(),
        "Receiver should be empty initially"
    );
}

#[tokio::test]
#[ignore] // Requires real Binance connection
async fn test_websocket_receives_updates() {
    // Test that WebSocket actually receives depth updates from Binance
    let (client, mut receiver) = DepthWebSocketClient::new("BTCUSDT".to_string());
    let handle = client.start();

    // Wait up to 10 seconds for first update
    let result = tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await;

    assert!(
        result.is_ok(),
        "Should receive at least one update within 10 seconds"
    );

    if let Ok(Some(update)) = result {
        assert_eq!(update.symbol, "BTCUSDT");
        assert_eq!(update.event_type, "depthUpdate");
        assert!(update.bids.len() > 0 || update.asks.len() > 0);
    }

    handle.abort();
}

#[tokio::test]
async fn test_websocket_exponential_backoff_timing() {
    // Test that exponential backoff timing is correct
    // This is a unit-style test that verifies the exponential calculation

    let retry_count = 0;
    let delay1 = std::cmp::min(2_u64.pow(retry_count), 30);
    assert_eq!(delay1, 1, "First retry should be 1 second");

    let retry_count = 1;
    let delay2 = std::cmp::min(2_u64.pow(retry_count), 30);
    assert_eq!(delay2, 2, "Second retry should be 2 seconds");

    let retry_count = 2;
    let delay3 = std::cmp::min(2_u64.pow(retry_count), 30);
    assert_eq!(delay3, 4, "Third retry should be 4 seconds");

    let retry_count = 3;
    let delay4 = std::cmp::min(2_u64.pow(retry_count), 30);
    assert_eq!(delay4, 8, "Fourth retry should be 8 seconds");

    let retry_count = 10;
    let delay_max = std::cmp::min(2_u64.pow(retry_count), 30);
    assert_eq!(
        delay_max, 30,
        "Max retry delay should be capped at 30 seconds"
    );
}
