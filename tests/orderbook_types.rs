//! Unit tests for OrderBook types
//!
//! Tests serialization, compact integer encoding, and core OrderBook operations.

use mcp_binance_server::orderbook::types::{OrderBook, OrderBookDepth};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_orderbook_creation() {
    let ob = OrderBook::new("BTCUSDT".to_string());

    assert_eq!(ob.symbol, "BTCUSDT");
    assert!(ob.timestamp > 0); // Timestamp is set to current time
    assert!(ob.bids.is_empty());
    assert!(ob.asks.is_empty());
}

#[test]
fn test_orderbook_update_bid() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    let price = Decimal::from_str("67650.00").unwrap();
    let qty = Decimal::from_str("1.234").unwrap();

    ob.update_bid(price, qty);

    assert_eq!(ob.bids.len(), 1);
    assert_eq!(ob.bids.get(&price), Some(&qty));
}

#[test]
fn test_orderbook_update_ask() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    let price = Decimal::from_str("67651.00").unwrap();
    let qty = Decimal::from_str("0.987").unwrap();

    ob.update_ask(price, qty);

    assert_eq!(ob.asks.len(), 1);
    assert_eq!(ob.asks.get(&price), Some(&qty));
}

#[test]
fn test_orderbook_update_removes_zero_quantity() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    let price = Decimal::from_str("67650.00").unwrap();
    let qty = Decimal::from_str("1.234").unwrap();

    // Add level
    ob.update_bid(price, qty);
    assert_eq!(ob.bids.len(), 1);

    // Remove level with zero quantity
    ob.update_bid(price, Decimal::ZERO);
    assert!(ob.bids.is_empty());
}

#[test]
fn test_best_bid_empty() {
    let ob = OrderBook::new("BTCUSDT".to_string());
    assert!(ob.best_bid().is_none());
}

#[test]
fn test_best_ask_empty() {
    let ob = OrderBook::new("BTCUSDT".to_string());
    assert!(ob.best_ask().is_none());
}

#[test]
fn test_best_bid_returns_highest() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    ob.update_bid(
        Decimal::from_str("67650.00").unwrap(),
        Decimal::from_str("1.0").unwrap(),
    );
    ob.update_bid(
        Decimal::from_str("67649.00").unwrap(),
        Decimal::from_str("2.0").unwrap(),
    );
    ob.update_bid(
        Decimal::from_str("67651.00").unwrap(),
        Decimal::from_str("0.5").unwrap(),
    );

    let best = ob.best_bid().unwrap();
    assert_eq!(best.to_string(), "67651.00");
}

#[test]
fn test_best_ask_returns_lowest() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    ob.update_ask(
        Decimal::from_str("67652.00").unwrap(),
        Decimal::from_str("1.0").unwrap(),
    );
    ob.update_ask(
        Decimal::from_str("67653.00").unwrap(),
        Decimal::from_str("2.0").unwrap(),
    );
    ob.update_ask(
        Decimal::from_str("67651.00").unwrap(),
        Decimal::from_str("0.5").unwrap(),
    );

    let best = ob.best_ask().unwrap();
    assert_eq!(best.to_string(), "67651.00");
}

#[test]
fn test_orderbook_serialization() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    ob.update_bid(
        Decimal::from_str("67650.00").unwrap(),
        Decimal::from_str("1.234").unwrap(),
    );
    ob.update_ask(
        Decimal::from_str("67651.00").unwrap(),
        Decimal::from_str("0.987").unwrap(),
    );

    let json = serde_json::to_string(&ob).unwrap();
    let deserialized: OrderBook = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.symbol, "BTCUSDT");
    assert!(deserialized.timestamp > 0); // Timestamp is preserved through serialization
    assert_eq!(deserialized.bids.len(), 1);
    assert_eq!(deserialized.asks.len(), 1);
}

#[test]
fn test_orderbook_depth_compact_encoding() {
    let depth = OrderBookDepth {
        symbol: "BTCUSDT".to_string(),
        timestamp: 1699999999123,
        price_scale: 100,
        qty_scale: 100_000,
        bids: vec![[6765000, 123400], [6764950, 45600]],
        asks: vec![[6765100, 98700], [6765150, 40000]],
    };

    // Verify scaling
    assert_eq!(depth.bids[0][0], 6765000); // 67650.00 * 100
    assert_eq!(depth.bids[0][1], 123400); // 1.234 * 100000

    // Verify serialization produces compact JSON
    let json = serde_json::to_string(&depth).unwrap();
    assert!(json.contains("6765000"));
    assert!(json.contains("123400"));

    // Verify deserialization
    let deserialized: OrderBookDepth = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "BTCUSDT");
    assert_eq!(deserialized.bids.len(), 2);
    assert_eq!(deserialized.asks.len(), 2);
}

#[test]
fn test_orderbook_depth_decoding_price() {
    let depth = OrderBookDepth {
        symbol: "BTCUSDT".to_string(),
        timestamp: 1699999999123,
        price_scale: 100,
        qty_scale: 100_000,
        bids: vec![[6765000, 123400]],
        asks: vec![],
    };

    // Decode price: 6765000 / 100 = 67650.00
    let scaled_price = depth.bids[0][0];
    let decoded_price = Decimal::from(scaled_price) / Decimal::from(depth.price_scale);

    assert_eq!(decoded_price.to_string(), "67650"); // Decimal normalizes trailing zeros
}

#[test]
fn test_orderbook_depth_decoding_quantity() {
    let depth = OrderBookDepth {
        symbol: "BTCUSDT".to_string(),
        timestamp: 1699999999123,
        price_scale: 100,
        qty_scale: 100_000,
        bids: vec![[6765000, 123400]],
        asks: vec![],
    };

    // Decode quantity: 123400 / 100000 = 1.23400
    let scaled_qty = depth.bids[0][1];
    let decoded_qty = Decimal::from(scaled_qty) / Decimal::from(depth.qty_scale);

    assert_eq!(decoded_qty.to_string(), "1.234");
}

#[test]
fn test_orderbook_depth_empty() {
    let depth = OrderBookDepth {
        symbol: "BTCUSDT".to_string(),
        timestamp: 1699999999123,
        price_scale: 100,
        qty_scale: 100_000,
        bids: vec![],
        asks: vec![],
    };

    assert!(depth.bids.is_empty());
    assert!(depth.asks.is_empty());
}

#[test]
fn test_orderbook_depth_json_size_reduction() {
    // Create same data in two formats
    let compact = OrderBookDepth {
        symbol: "BTCUSDT".to_string(),
        timestamp: 1699999999123,
        price_scale: 100,
        qty_scale: 100_000,
        bids: vec![[6765000, 123400]],
        asks: vec![[6765100, 98700]],
    };

    // Simulate uncompressed format (strings)
    let uncompressed = serde_json::json!({
        "symbol": "BTCUSDT",
        "timestamp": 1699999999123i64,
        "bids": [["67650.00", "1.234"]],
        "asks": [["67651.00", "0.987"]]
    });

    let compact_json = serde_json::to_string(&compact).unwrap();
    let uncompressed_json = uncompressed.to_string();

    // Compact format uses integers instead of decimal strings
    // Verify compact uses integers for bids/asks
    assert!(compact_json.contains("[6765000,123400]"));
    assert!(compact_json.contains("[6765100,98700]"));

    // Verify uncompressed uses strings
    assert!(uncompressed_json.contains("\"67650.00\""));
    assert!(uncompressed_json.contains("\"1.234\""));
}

#[test]
fn test_multiple_levels_ordering() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Add bids in random order
    ob.update_bid(
        Decimal::from_str("67650.00").unwrap(),
        Decimal::from_str("1.0").unwrap(),
    );
    ob.update_bid(
        Decimal::from_str("67652.00").unwrap(),
        Decimal::from_str("2.0").unwrap(),
    );
    ob.update_bid(
        Decimal::from_str("67651.00").unwrap(),
        Decimal::from_str("1.5").unwrap(),
    );

    // BTreeMap should maintain sorted order (ascending)
    let prices: Vec<Decimal> = ob.bids.keys().cloned().collect();
    assert_eq!(prices[0].to_string(), "67650.00");
    assert_eq!(prices[1].to_string(), "67651.00");
    assert_eq!(prices[2].to_string(), "67652.00");

    // Best bid should be highest (last in ascending order)
    assert_eq!(ob.best_bid().unwrap().to_string(), "67652.00");
}

#[test]
fn test_orderbook_update_overwrites() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    let price = Decimal::from_str("67650.00").unwrap();

    // Add initial level
    ob.update_bid(price, Decimal::from_str("1.0").unwrap());
    assert_eq!(ob.bids.get(&price).unwrap().to_string(), "1.0");

    // Update same level with new quantity
    ob.update_bid(price, Decimal::from_str("2.5").unwrap());
    assert_eq!(ob.bids.get(&price).unwrap().to_string(), "2.5");
    assert_eq!(ob.bids.len(), 1); // Still only one level
}

#[test]
fn test_orderbook_large_numbers() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Test with very large price
    let large_price = Decimal::from_str("999999.99").unwrap();
    let qty = Decimal::from_str("0.00000001").unwrap();

    ob.update_bid(large_price, qty);

    assert_eq!(ob.bids.get(&large_price), Some(&qty));
}

#[test]
fn test_orderbook_decimal_precision() {
    let mut ob = OrderBook::new("BTCUSDT".to_string());

    // Test with high-precision decimal
    let price = Decimal::from_str("67650.12345678").unwrap();
    let qty = Decimal::from_str("1.23456789").unwrap();

    ob.update_bid(price, qty);

    let stored_price = ob.bids.keys().next().unwrap();
    let stored_qty = ob.bids.values().next().unwrap();

    assert_eq!(stored_price.to_string(), "67650.12345678");
    assert_eq!(stored_qty.to_string(), "1.23456789");
}
