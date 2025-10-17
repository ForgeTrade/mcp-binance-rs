//! Account Information API integration tests
//!
//! Tests for 5 authenticated account endpoints:
//! - GET /api/v3/account - Account information (balances, permissions)
//! - GET /api/v3/myTrades - Account trade history
//! - GET /api/v3/rateLimit/order - Current rate limit usage
//! - POST /api/v3/userDataStream - Start user data stream
//! - DELETE /api/v3/userDataStream - Close user data stream

use super::{
    assert_cors_headers, build_authenticated_url, setup_authenticated_client, wait_for_rate_limit,
};
use crate::common::{assertions, init_test_env};

/// T024: Test GET /api/v3/account endpoint
/// Returns account information including balances and permissions
#[tokio::test]
async fn test_account_info() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    let url = build_authenticated_url(&creds.base_url, "account", &[], &creds.api_secret);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send account info request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse and validate JSON schema
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse account info JSON");

    assertions::assert_account_schema(&json);

    // Verify balances array structure
    let balances = json["balances"]
        .as_array()
        .expect("balances should be array");

    for balance in balances {
        assertions::assert_has_fields(balance, &["asset", "free", "locked"]);
        assertions::assert_field_type(balance, "asset", assertions::JsonType::String);
        assertions::assert_field_type(balance, "free", assertions::JsonType::String);
        assertions::assert_field_type(balance, "locked", assertions::JsonType::String);
    }

    wait_for_rate_limit().await;
}

/// T025: Test GET /api/v3/myTrades endpoint
/// Returns account trade history for a symbol
#[tokio::test]
async fn test_my_trades() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    let url = build_authenticated_url(
        &creds.base_url,
        "myTrades",
        &[("symbol", "BTCUSDT"), ("limit", "10")],
        &creds.api_secret,
    );

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send my trades request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse JSON array
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse my trades JSON");

    let trades = json.as_array().expect("Expected array of trades");

    assert!(trades.len() <= 10, "Trades should not exceed limit of 10");

    // Verify each trade has required fields
    for trade in trades {
        assertions::assert_has_fields(
            trade,
            &[
                "id",
                "orderId",
                "symbol",
                "price",
                "qty",
                "commission",
                "commissionAsset",
                "time",
                "isBuyer",
                "isMaker",
            ],
        );

        assertions::assert_field_type(trade, "id", assertions::JsonType::Number);
        assertions::assert_field_type(trade, "symbol", assertions::JsonType::String);
        assertions::assert_field_type(trade, "price", assertions::JsonType::String);
        assertions::assert_field_type(trade, "isBuyer", assertions::JsonType::Boolean);
        assertions::assert_field_type(trade, "isMaker", assertions::JsonType::Boolean);

        // Verify symbol matches request
        assert_eq!(
            trade["symbol"].as_str().unwrap(),
            "BTCUSDT",
            "Trade symbol should match request"
        );
    }

    wait_for_rate_limit().await;
}

/// T026: Test GET /api/v3/rateLimit/order endpoint
/// Returns current API rate limit usage
#[tokio::test]
async fn test_rate_limit_info() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    let url = build_authenticated_url(&creds.base_url, "rateLimit/order", &[], &creds.api_secret);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send rate limit request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse JSON array of rate limiters
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse rate limit JSON");

    let rate_limiters = json.as_array().expect("Expected array of rate limiters");

    // Verify each rate limiter has required fields
    for limiter in rate_limiters {
        assertions::assert_has_fields(
            limiter,
            &["rateLimitType", "interval", "intervalNum", "limit"],
        );

        assertions::assert_field_type(limiter, "rateLimitType", assertions::JsonType::String);
        assertions::assert_field_type(limiter, "interval", assertions::JsonType::String);
        assertions::assert_field_type(limiter, "intervalNum", assertions::JsonType::Number);
        assertions::assert_field_type(limiter, "limit", assertions::JsonType::Number);

        // Verify limit is positive
        let limit = limiter["limit"].as_i64().expect("limit should be number");
        assert!(limit > 0, "Rate limit should be positive");
    }

    wait_for_rate_limit().await;
}

/// T027: Test POST /api/v3/userDataStream endpoint
/// Starts a new user data stream and returns listenKey
#[tokio::test]
async fn test_start_user_data_stream() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    let url = format!("{}/api/v3/userDataStream", creds.base_url);

    let response = client
        .post(&url)
        .send()
        .await
        .expect("Failed to send start user data stream request");

    // Assert successful response (200 or 201)
    let status = response.status();
    assert!(
        status == 200 || status == 201,
        "Expected 200/201 status, got {}",
        status
    );

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse JSON response
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse user data stream JSON");

    // Verify listenKey is present
    assertions::assert_has_fields(&json, &["listenKey"]);
    assertions::assert_field_type(&json, "listenKey", assertions::JsonType::String);

    let listen_key = json["listenKey"]
        .as_str()
        .expect("listenKey should be string");
    assert!(!listen_key.is_empty(), "listenKey should not be empty");

    // Verify listenKey format (typically 60 characters alphanumeric)
    assert!(
        listen_key.len() >= 40,
        "listenKey should be at least 40 characters"
    );
    assert!(
        listen_key.chars().all(|c| c.is_alphanumeric()),
        "listenKey should be alphanumeric"
    );

    wait_for_rate_limit().await;
}

/// T028: Test DELETE /api/v3/userDataStream endpoint
/// Closes an existing user data stream
#[tokio::test]
async fn test_close_user_data_stream() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    // First, start a user data stream to get a listenKey
    let start_url = format!("{}/api/v3/userDataStream", creds.base_url);

    let start_response = client
        .post(&start_url)
        .send()
        .await
        .expect("Failed to start user data stream");

    let start_json: serde_json::Value = start_response
        .json()
        .await
        .expect("Failed to parse start stream JSON");

    let listen_key = start_json["listenKey"]
        .as_str()
        .expect("listenKey should be string");

    wait_for_rate_limit().await;

    // Now close the stream
    let close_url = format!(
        "{}/api/v3/userDataStream?listenKey={}",
        creds.base_url, listen_key
    );

    let response = client
        .delete(&close_url)
        .send()
        .await
        .expect("Failed to send close user data stream request");

    // Assert successful response (200 or 204 No Content)
    let status = response.status();
    assert!(
        status == 200 || status == 204,
        "Expected 200/204 status, got {}",
        status
    );

    // Assert CORS headers present
    assert_cors_headers(&response);

    // For 200 status, verify empty JSON object response
    if status == 200 {
        let json: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse close stream JSON");

        // Binance typically returns {} for successful DELETE
        assert!(json.is_object(), "Response should be empty JSON object");
    }

    wait_for_rate_limit().await;
}
