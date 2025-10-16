//! get_server_time Tool Integration Tests
//!
//! Tests the get_server_time tool behavior directly via Binance client.

use mcp_binance_server::server::BinanceServer;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_get_server_time_via_client() {
    let server = BinanceServer::new();

    // Call get_server_time directly through the internal client
    let result = server.binance_client.get_server_time().await;

    assert!(result.is_ok(), "get_server_time should succeed");

    let server_time = result.unwrap();
    assert!(server_time > 0, "Server time should be positive");

    // Verify time is within reasonable range (±60 seconds of local time)
    let local_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let time_diff = (server_time - local_time).abs();
    assert!(
        time_diff < 60_000,
        "Server time should be within 60s of local time. Diff: {}ms",
        time_diff
    );
}

#[tokio::test]
async fn test_multiple_sequential_calls() {
    let server = BinanceServer::new();

    // Make multiple calls to verify no state corruption
    for i in 0..5 {
        let result = server.binance_client.get_server_time().await;

        assert!(
            result.is_ok(),
            "Call {} should succeed: {:?}",
            i + 1,
            result
        );

        let server_time = result.unwrap();
        assert!(server_time > 0);
    }
}

#[tokio::test]
async fn test_concurrent_calls() {
    let server = BinanceServer::new();

    // Make concurrent calls
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let client = server.binance_client.clone();
            tokio::spawn(async move { client.get_server_time().await })
        })
        .collect();

    // Wait for all calls to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent call {} should succeed", i + 1);
    }
}

#[tokio::test]
async fn test_time_synchronization() {
    // This test verifies the client can connect and get time
    // Actual time offset warnings are logged, not returned
    let server = BinanceServer::new();

    let result = server.binance_client.get_server_time().await;

    assert!(result.is_ok(), "Should handle time sync gracefully");
}

#[tokio::test]
async fn test_server_time_monotonic() {
    let server = BinanceServer::new();

    // Get time twice
    let time1 = server
        .binance_client
        .get_server_time()
        .await
        .expect("First call should succeed");

    // Small delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let time2 = server
        .binance_client
        .get_server_time()
        .await
        .expect("Second call should succeed");

    // Second time should be greater than or equal to first
    assert!(
        time2 >= time1,
        "Time should be monotonic: time1={}, time2={}",
        time1,
        time2
    );
}
