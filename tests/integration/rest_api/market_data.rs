//! Market Data API integration tests
//!
//! Tests for 5 public market data endpoints:
//! - GET /api/v3/ticker/24hr - 24-hour ticker price change statistics
//! - GET /api/v3/depth - Order book depth (bids/asks)
//! - GET /api/v3/trades - Recent trades list
//! - GET /api/v3/klines - Candlestick/kline data
//! - GET /api/v3/avgPrice - Current average price

use super::{assert_cors_headers, setup_public_client, wait_for_rate_limit};
use crate::common::{assertions, fixtures::TestCredentials, init_test_env};

/// T014: Test GET /api/v3/ticker/24hr endpoint
/// Returns 24-hour price change statistics for a symbol
#[tokio::test]
async fn test_ticker_24hr() {
    init_test_env();

    let client = setup_public_client();
    let creds = TestCredentials::from_env();

    let url = format!("{}/api/v3/ticker/24hr?symbol=BTCUSDT", creds.base_url);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send ticker request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present (T028a)
    assert_cors_headers(&response);

    // Parse and validate JSON schema
    let json: serde_json::Value = response.json().await.expect("Failed to parse ticker JSON");

    assertions::assert_ticker_schema(&json);

    // Verify symbol matches request
    assert_eq!(
        json["symbol"].as_str().unwrap(),
        "BTCUSDT",
        "Symbol should match request"
    );

    wait_for_rate_limit().await;
}

/// T015: Test GET /api/v3/depth endpoint
/// Returns order book depth with bids and asks
#[tokio::test]
async fn test_order_book_depth() {
    init_test_env();

    let client = setup_public_client();
    let creds = TestCredentials::from_env();

    let url = format!("{}/api/v3/depth?symbol=BTCUSDT&limit=10", creds.base_url);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send depth request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present (T028a)
    assert_cors_headers(&response);

    // Parse and validate JSON schema
    let json: serde_json::Value = response.json().await.expect("Failed to parse depth JSON");

    assertions::assert_depth_schema(&json);

    // Verify depth limit matches request
    let bids = json["bids"].as_array().expect("bids should be array");
    let asks = json["asks"].as_array().expect("asks should be array");

    assert!(bids.len() <= 10, "Bids should not exceed limit");
    assert!(asks.len() <= 10, "Asks should not exceed limit");

    wait_for_rate_limit().await;
}

/// T016: Test GET /api/v3/trades endpoint
/// Returns recent trades list for a symbol
#[tokio::test]
async fn test_recent_trades() {
    init_test_env();

    let client = setup_public_client();
    let creds = TestCredentials::from_env();

    let url = format!("{}/api/v3/trades?symbol=BTCUSDT&limit=5", creds.base_url);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send trades request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present (T028a)
    assert_cors_headers(&response);

    // Parse JSON array
    let json: serde_json::Value = response.json().await.expect("Failed to parse trades JSON");

    let trades = json.as_array().expect("Expected array of trades");
    assert!(trades.len() <= 5, "Trades should not exceed limit");

    // Validate first trade has required fields
    if let Some(first_trade) = trades.first() {
        assertions::assert_has_fields(first_trade, &["id", "price", "qty", "time", "isBuyerMaker"]);

        assertions::assert_field_type(first_trade, "price", assertions::JsonType::String);
        assertions::assert_field_type(first_trade, "qty", assertions::JsonType::String);
        assertions::assert_field_type(first_trade, "time", assertions::JsonType::Number);
        assertions::assert_field_type(first_trade, "isBuyerMaker", assertions::JsonType::Boolean);
    }

    wait_for_rate_limit().await;
}

/// T017: Test GET /api/v3/klines endpoint
/// Returns candlestick/kline data for a symbol
#[tokio::test]
async fn test_klines() {
    init_test_env();

    let client = setup_public_client();
    let creds = TestCredentials::from_env();

    let url = format!(
        "{}/api/v3/klines?symbol=BTCUSDT&interval=1h&limit=5",
        creds.base_url
    );

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send klines request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present (T028a)
    assert_cors_headers(&response);

    // Parse JSON array
    let json: serde_json::Value = response.json().await.expect("Failed to parse klines JSON");

    let klines = json.as_array().expect("Expected array of klines");
    assert!(klines.len() <= 5, "Klines should not exceed limit");

    // Validate kline structure: [time, open, high, low, close, volume, ...]
    if let Some(first_kline) = klines.first() {
        let kline_arr = first_kline.as_array().expect("Each kline should be array");
        assert!(
            kline_arr.len() >= 6,
            "Kline should have at least 6 elements [time, open, high, low, close, volume]"
        );

        // Verify first element is timestamp (number)
        assert!(
            kline_arr[0].is_number(),
            "Kline[0] should be timestamp (number)"
        );

        // Verify OHLCV values are strings
        for (idx, value) in kline_arr.iter().enumerate().take(6).skip(1) {
            assert!(value.is_string(), "Kline[{}] should be string (OHLCV)", idx);
        }
    }

    wait_for_rate_limit().await;
}

/// T018: Test GET /api/v3/avgPrice endpoint
/// Returns current average price for a symbol
#[tokio::test]
async fn test_average_price() {
    init_test_env();

    let client = setup_public_client();
    let creds = TestCredentials::from_env();

    let url = format!("{}/api/v3/avgPrice?symbol=BTCUSDT", creds.base_url);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send avgPrice request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present (T028a)
    assert_cors_headers(&response);

    // Parse and validate JSON schema
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse avgPrice JSON");

    assertions::assert_has_fields(&json, &["mins", "price"]);

    assertions::assert_field_type(&json, "mins", assertions::JsonType::Number);
    assertions::assert_field_type(&json, "price", assertions::JsonType::String);

    // Verify price is positive
    let price_str = json["price"].as_str().expect("price should be string");
    let price: f64 = price_str.parse().expect("price should be valid number");
    assert!(price > 0.0, "Average price should be positive");

    wait_for_rate_limit().await;
}
