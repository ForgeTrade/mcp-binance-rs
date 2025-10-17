//! Performance tests for orderbook depth tools
//!
//! Measures L1 metrics and L2 depth query latency.

#[cfg(feature = "orderbook")]
mod orderbook_performance {
    use mcp_binance_server::orderbook::manager::OrderBookManager;
    use mcp_binance_server::orderbook::metrics::{calculate_metrics, extract_depth};
    use mcp_binance_server::orderbook::types::OrderBook;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Instant;

    /// Create a realistic order book with multiple levels
    fn create_test_orderbook() -> OrderBook {
        let mut ob = OrderBook::new("BTCUSDT".to_string());

        // Add 100 bid levels
        for i in 0..100 {
            let price = 67650.0 - (i as f64) * 0.50;
            let qty = 1.0 + ((i % 10) as f64) * 0.1;
            ob.update_bid(
                Decimal::from_str(&price.to_string()).unwrap(),
                Decimal::from_str(&qty.to_string()).unwrap(),
            );
        }

        // Add 100 ask levels
        for i in 0..100 {
            let price = 67651.0 + (i as f64) * 0.50;
            let qty = 1.0 + ((i % 10) as f64) * 0.1;
            ob.update_ask(
                Decimal::from_str(&price.to_string()).unwrap(),
                Decimal::from_str(&qty.to_string()).unwrap(),
            );
        }

        ob
    }

    #[test]
    fn test_l1_metrics_performance() {
        // Test L1 metrics calculation performance (P95 ≤200ms target)
        let ob = create_test_orderbook();
        let mut latencies = Vec::new();

        // Warm up
        for _ in 0..10 {
            let _ = calculate_metrics(&ob);
        }

        // Measure 100 iterations
        for _ in 0..100 {
            let start = Instant::now();
            let _ = calculate_metrics(&ob);
            latencies.push(start.elapsed().as_micros());
        }

        // Calculate P95
        latencies.sort_unstable();
        let p95_idx = (latencies.len() as f64 * 0.95) as usize;
        let p95_latency_us = latencies[p95_idx];
        let p95_latency_ms = p95_latency_us as f64 / 1000.0;

        println!(
            "L1 metrics performance: P95={:.2}ms, min={:.2}ms, max={:.2}ms",
            p95_latency_ms,
            latencies[0] as f64 / 1000.0,
            latencies[latencies.len() - 1] as f64 / 1000.0
        );

        assert!(
            p95_latency_ms <= 200.0,
            "L1 metrics P95 latency should be ≤200ms, got {:.2}ms",
            p95_latency_ms
        );
    }

    #[test]
    fn test_l2_depth_performance() {
        // Test L2 depth extraction performance (≤300ms target)
        let ob = create_test_orderbook();
        let mut latencies = Vec::new();

        // Warm up
        for _ in 0..10 {
            let _ = extract_depth(&ob, 100);
        }

        // Measure 100 iterations
        for _ in 0..100 {
            let start = Instant::now();
            let _ = extract_depth(&ob, 100);
            latencies.push(start.elapsed().as_micros());
        }

        // Calculate P95
        latencies.sort_unstable();
        let p95_idx = (latencies.len() as f64 * 0.95) as usize;
        let p95_latency_us = latencies[p95_idx];
        let p95_latency_ms = p95_latency_us as f64 / 1000.0;

        println!(
            "L2 depth performance: P95={:.2}ms, min={:.2}ms, max={:.2}ms",
            p95_latency_ms,
            latencies[0] as f64 / 1000.0,
            latencies[latencies.len() - 1] as f64 / 1000.0
        );

        assert!(
            p95_latency_ms <= 300.0,
            "L2 depth P95 latency should be ≤300ms, got {:.2}ms",
            p95_latency_ms
        );
    }

    #[test]
    fn test_l2_lite_performance() {
        // Test L2-lite (20 levels) performance
        let ob = create_test_orderbook();
        let mut latencies = Vec::new();

        // Warm up
        for _ in 0..10 {
            let _ = extract_depth(&ob, 20);
        }

        // Measure 100 iterations
        for _ in 0..100 {
            let start = Instant::now();
            let _ = extract_depth(&ob, 20);
            latencies.push(start.elapsed().as_micros());
        }

        // Calculate average
        let avg_latency_us: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;
        let avg_latency_ms = avg_latency_us as f64 / 1000.0;

        println!("L2-lite (20 levels) average latency: {:.2}ms", avg_latency_ms);

        // L2-lite should be faster than L2-full
        assert!(
            avg_latency_ms < 100.0,
            "L2-lite should be fast, got {:.2}ms",
            avg_latency_ms
        );
    }

    #[tokio::test]
    async fn test_manager_creation_performance() {
        // Test OrderBookManager creation performance
        let client = Arc::new(mcp_binance_server::binance::BinanceClient::new());

        let start = Instant::now();
        let _manager = OrderBookManager::new(client);
        let elapsed = start.elapsed();

        println!(
            "OrderBookManager creation time: {:.2}ms",
            elapsed.as_micros() as f64 / 1000.0
        );

        assert!(
            elapsed.as_millis() < 500,
            "Manager creation should be fast, took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_orderbook_update_performance() {
        // Test order book update operation performance
        let mut ob = OrderBook::new("BTCUSDT".to_string());
        let mut latencies = Vec::new();

        // Pre-populate with some levels
        for i in 0..50 {
            ob.update_bid(
                Decimal::from_str(&format!("{}", 67650 - i)).unwrap(),
                Decimal::from_str("1.0").unwrap(),
            );
        }

        // Measure 1000 update operations
        for i in 0..1000 {
            let price = Decimal::from_str(&format!("{}", 67600 + (i % 100))).unwrap();
            let qty = Decimal::from_str("1.5").unwrap();

            let start = Instant::now();
            ob.update_bid(price, qty);
            latencies.push(start.elapsed().as_nanos());
        }

        let avg_latency_ns: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;

        println!(
            "OrderBook update average latency: {:.2}µs",
            avg_latency_ns as f64 / 1000.0
        );

        // Updates should be very fast (< 100 microseconds)
        assert!(
            avg_latency_ns < 100_000,
            "OrderBook updates should be fast, got {}ns",
            avg_latency_ns
        );
    }
}
