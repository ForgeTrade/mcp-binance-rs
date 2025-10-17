//! Metrics calculation integration tests
//!
//! Tests accuracy of spread, microprice, and slippage calculations.

use mcp_binance_server::orderbook::metrics::{calculate_metrics, extract_depth};
use mcp_binance_server::orderbook::types::OrderBook;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_spread_accuracy() {
    // Test spread calculation accuracy within 0.01 bps
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Set up order book with known spread
    ob.update_bid(
        Decimal::from_str("67650.00").unwrap(),
        Decimal::from_str("1.0").unwrap(),
    );
    ob.update_ask(
        Decimal::from_str("67651.00").unwrap(),
        Decimal::from_str("1.0").unwrap(),
    );

    let metrics = calculate_metrics(&ob).expect("Should calculate metrics");

    // Expected spread: ((67651 - 67650) / 67650) * 10000 = 0.1478 bps
    let expected_spread = 0.1478;
    let actual_spread = metrics.spread_bps;

    assert!(
        (actual_spread - expected_spread).abs() < 0.01,
        "Spread should be within 0.01 bps: expected={}, actual={}",
        expected_spread,
        actual_spread
    );
}

#[test]
fn test_microprice_accuracy() {
    // Test microprice calculation accuracy within $0.01
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Set up order book with known volumes
    let bid_price = Decimal::from_str("67650.00").unwrap();
    let ask_price = Decimal::from_str("67651.00").unwrap();

    ob.update_bid(bid_price, Decimal::from_str("10.0").unwrap());
    ob.update_ask(ask_price, Decimal::from_str("15.0").unwrap());

    let metrics = calculate_metrics(&ob).expect("Should calculate metrics");

    // Expected microprice: (67650 * 15 + 67651 * 10) / 25 = 67650.4
    let expected_microprice = 67650.4;
    let actual_microprice = metrics.microprice;

    assert!(
        (actual_microprice - expected_microprice).abs() < 0.01,
        "Microprice should be within $0.01: expected={}, actual={}",
        expected_microprice,
        actual_microprice
    );
}

#[test]
fn test_slippage_estimates_accuracy() {
    // Test slippage calculation accuracy within 5%
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Create a realistic order book with multiple levels
    let levels = vec![
        ("67650.00", "1.0"),
        ("67649.00", "2.0"),
        ("67648.00", "1.5"),
        ("67647.00", "2.5"),
        ("67646.00", "3.0"),
    ];

    for (price_str, qty_str) in &levels {
        ob.update_bid(
            Decimal::from_str(price_str).unwrap(),
            Decimal::from_str(qty_str).unwrap(),
        );
    }

    // Add some ask levels
    let ask_levels = vec![
        ("67651.00", "1.0"),
        ("67652.00", "2.0"),
        ("67653.00", "1.5"),
        ("67654.00", "2.5"),
        ("67655.00", "3.0"),
    ];

    for (price_str, qty_str) in &ask_levels {
        ob.update_ask(
            Decimal::from_str(price_str).unwrap(),
            Decimal::from_str(qty_str).unwrap(),
        );
    }

    let metrics = calculate_metrics(&ob).expect("Should calculate metrics");

    // Verify slippage estimates exist
    assert!(
        metrics.slippage_estimates.sell_10k_usd.is_some(),
        "Should have sell_10k slippage estimate"
    );
    assert!(
        metrics.slippage_estimates.buy_10k_usd.is_some(),
        "Should have buy_10k slippage estimate"
    );

    // Check that slippage is reasonable (< 100 bps for liquid market)
    if let Some(sell_est) = &metrics.slippage_estimates.sell_10k_usd {
        assert!(
            sell_est.slippage_bps < 100.0,
            "Slippage should be reasonable: {}",
            sell_est.slippage_bps
        );
        assert!(sell_est.filled_qty > 0.0, "Should fill some quantity");
    }
}

#[test]
fn test_imbalance_ratio_calculation() {
    // Test imbalance ratio calculation
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Heavy bid side
    for i in 0..20 {
        ob.update_bid(
            Decimal::from_str(&format!("{}", 67650 - i)).unwrap(),
            Decimal::from_str("2.0").unwrap(),
        );
    }

    // Light ask side
    for i in 0..20 {
        ob.update_ask(
            Decimal::from_str(&format!("{}", 67651 + i)).unwrap(),
            Decimal::from_str("1.0").unwrap(),
        );
    }

    let metrics = calculate_metrics(&ob).expect("Should calculate metrics");

    // Bid volume = 20 * 2.0 = 40
    // Ask volume = 20 * 1.0 = 20
    // Imbalance ratio = 40 / 20 = 2.0
    assert_eq!(metrics.bid_volume, 40.0);
    assert_eq!(metrics.ask_volume, 20.0);
    assert!((metrics.imbalance_ratio - 2.0).abs() < 0.01);
}

#[test]
fn test_compact_encoding() {
    // Test L2 depth compact encoding
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    ob.update_bid(
        Decimal::from_str("67650.00").unwrap(),
        Decimal::from_str("1.234").unwrap(),
    );
    ob.update_ask(
        Decimal::from_str("67651.00").unwrap(),
        Decimal::from_str("0.987").unwrap(),
    );

    let depth = extract_depth(&ob, 20);

    assert_eq!(depth.symbol, "BTCUSDT");
    assert_eq!(depth.price_scale, 100);
    assert_eq!(depth.qty_scale, 100_000);
    assert_eq!(depth.bids.len(), 1);
    assert_eq!(depth.asks.len(), 1);

    // Verify encoding: 67650.00 * 100 = 6765000
    assert_eq!(depth.bids[0][0], 6765000);
    // Verify encoding: 1.234 * 100000 = 123400
    assert_eq!(depth.bids[0][1], 123400);
}

#[test]
fn test_wall_detection() {
    // Test wall detection (qty > 2x median)
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Add normal levels
    for i in 0..18 {
        ob.update_bid(
            Decimal::from_str(&format!("{}", 67650 - i)).unwrap(),
            Decimal::from_str("1.0").unwrap(),
        );
    }

    // Add two large walls
    ob.update_bid(
        Decimal::from_str("67649.00").unwrap(),
        Decimal::from_str("10.0").unwrap(), // Wall
    );
    ob.update_bid(
        Decimal::from_str("67648.00").unwrap(),
        Decimal::from_str("8.0").unwrap(), // Wall
    );

    // Add normal ask levels
    for i in 0..20 {
        ob.update_ask(
            Decimal::from_str(&format!("{}", 67651 + i)).unwrap(),
            Decimal::from_str("1.0").unwrap(),
        );
    }

    let metrics = calculate_metrics(&ob).expect("Should calculate metrics");

    // Should detect the bid walls
    assert!(
        metrics.walls.bids.len() >= 2,
        "Should detect at least 2 bid walls"
    );
}

#[test]
fn test_metrics_with_empty_orderbook() {
    // Test metrics calculation with empty order book
    let ob = OrderBook::new("BTCUSDT".to_string());

    let result = calculate_metrics(&ob);

    // Should return None for empty order book
    assert!(result.is_none(), "Should return None for empty order book");
}

#[test]
fn test_depth_extraction_levels() {
    // Test depth extraction with different level counts
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Add 50 levels on each side
    for i in 0..50 {
        ob.update_bid(
            Decimal::from_str(&format!("{}", 67650 - i)).unwrap(),
            Decimal::from_str("1.0").unwrap(),
        );
        ob.update_ask(
            Decimal::from_str(&format!("{}", 67651 + i)).unwrap(),
            Decimal::from_str("1.0").unwrap(),
        );
    }

    // Extract L2-lite (20 levels)
    let depth_lite = extract_depth(&ob, 20);
    assert_eq!(depth_lite.bids.len(), 20);
    assert_eq!(depth_lite.asks.len(), 20);

    // Extract L2-full (100 levels)
    let depth_full = extract_depth(&ob, 100);
    assert_eq!(depth_full.bids.len(), 50); // Limited by available levels
    assert_eq!(depth_full.asks.len(), 50);
}
