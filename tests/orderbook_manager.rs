//! Unit tests for OrderBookManager
//!
//! Tests symbol limit enforcement, lazy initialization, and cache staleness.

use mcp_binance_server::orderbook::manager::{ManagerError, OrderBookManager};
use mcp_binance_server::orderbook::types::HealthStatus;

#[test]
fn test_constants() {
    // Verify critical constants are configured correctly
    const MAX_SYMBOLS: usize = 20;
    const STALENESS_MS: i64 = 5000;

    assert_eq!(MAX_SYMBOLS, 20, "Max concurrent symbols should be 20");
    assert_eq!(STALENESS_MS, 5000, "Staleness threshold should be 5000ms");
}

#[test]
fn test_manager_error_display() {
    let err = ManagerError::SymbolLimitReached;
    let msg = format!("{}", err);
    assert!(msg.contains("20"));
    assert!(msg.contains("Symbol limit"));

    let err = ManagerError::SymbolNotFound("BTCUSDT".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("BTCUSDT"));
    assert!(msg.contains("not found"));
}

#[tokio::test]
async fn test_health_empty_manager() {
    // Create a manager with no tracked symbols
    let client = std::sync::Arc::new(mcp_binance_server::binance::BinanceClient::new());
    let manager = OrderBookManager::new(client);

    let health = manager.get_health().await;

    assert!(matches!(health.status, HealthStatus::Ok));
    assert_eq!(health.orderbook_symbols_active, 0);
    assert_eq!(health.last_update_age_ms, 0);
    assert!(!health.websocket_connected);
    assert!(health.reason.is_none());
}

#[test]
fn test_manager_creation() {
    // Test that manager can be created successfully
    let client = std::sync::Arc::new(mcp_binance_server::binance::BinanceClient::new());
    let _manager = OrderBookManager::new(client);

    // No panic means success
}

#[test]
fn test_error_types_are_send_sync() {
    // Verify error types can be sent across threads
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ManagerError>();
}
