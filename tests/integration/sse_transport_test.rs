//! Integration tests for SSE transport (Feature 009 - Phase 3)
//!
//! These tests verify SSE handshake, message exchange, and tool calls over SSE transport.
//! Tests follow test-first approach: written BEFORE implementation (T015-T019).
//!
//! ## Test Coverage
//!
//! - T016: SSE handshake establishes connection and returns connection-id header
//! - T017: POST to /mcp/message with valid connection-id returns 202 Accepted
//! - T018: Call `get_ticker` via SSE returns valid ticker data within 2s
//! - T019: 3 concurrent SSE connections all succeed and receive unique connection IDs
//!
//! ## Running Tests
//!
//! ```bash
//! cargo test --features sse,orderbook_analytics sse_transport
//! ```

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceExt;

/// Helper function to create test SSE server router
///
/// Returns configured axum Router with SSE endpoints for testing.
/// Uses test configuration with short timeouts and low connection limits.
async fn create_test_sse_router() -> axum::Router {
    use mcp_binance_server::server::BinanceServer;
    use mcp_binance_server::transport::sse::{SessionManager, SseState, message_post};

    let session_manager = SessionManager::new();
    let mcp_server = BinanceServer::new();
    let state = SseState::new(session_manager, mcp_server);

    // Create router with Streamable HTTP endpoint (March 2025 spec)
    axum::Router::new()
        .route("/mcp", axum::routing::post(message_post))
        .with_state(state)
}

/// T016: Test SSE handshake establishes connection and returns connection-id header
///
/// ## Acceptance Criteria
///
/// - GET /mcp/sse returns 200 OK
/// - Response headers include X-Connection-ID
/// - Connection ID is valid UUID v4 format
/// - Response is SSE stream (Content-Type: text/event-stream)
#[tokio::test]
async fn test_sse_handshake_returns_connection_id() {
    let app = create_test_sse_router().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mcp/sse")
                .header("Accept", "text/event-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert HTTP 200 OK
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "SSE handshake should return 200 OK"
    );

    // Assert X-Connection-ID header exists
    let connection_id = response
        .headers()
        .get("X-Connection-ID")
        .expect("Response should include X-Connection-ID header");

    let connection_id_str = connection_id.to_str().unwrap();

    // Assert connection ID is valid UUID v4 format
    assert!(
        uuid::Uuid::parse_str(connection_id_str).is_ok(),
        "Connection ID should be valid UUID v4: {}",
        connection_id_str
    );

    // Assert Content-Type is text/event-stream
    let content_type = response
        .headers()
        .get("Content-Type")
        .expect("Response should include Content-Type header");

    assert_eq!(
        content_type,
        "text/event-stream",
        "SSE response should have Content-Type: text/event-stream"
    );
}

/// T017: Test POST to /mcp/message with valid connection-id returns 202 Accepted
///
/// ## Acceptance Criteria
///
/// - POST /mcp/message with valid connection-id returns 202 Accepted
/// - Invalid connection-id returns 404 Not Found
/// - Missing connection-id returns 400 Bad Request
/// - Request body must be valid JSON-RPC 2.0
#[tokio::test]
async fn test_post_message_with_valid_connection_id() {
    let app = create_test_sse_router().await;

    // First, establish SSE connection to get connection ID
    let handshake_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mcp/sse")
                .header("Accept", "text/event-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let connection_id = handshake_response
        .headers()
        .get("X-Connection-ID")
        .expect("Handshake should return connection ID")
        .to_str()
        .unwrap();

    // Test valid connection ID with JSON-RPC request
    let json_rpc_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/message")
                .header("Content-Type", "application/json")
                .header("X-Connection-ID", connection_id)
                .body(Body::from(serde_json::to_string(&json_rpc_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "POST /mcp/message with valid connection-id should return 202 Accepted"
    );

    // Test invalid connection ID returns 404
    let invalid_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/message")
                .header("Content-Type", "application/json")
                .header("X-Connection-ID", "invalid-uuid")
                .body(Body::from(serde_json::to_string(&json_rpc_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        invalid_response.status(),
        StatusCode::NOT_FOUND,
        "POST /mcp/message with invalid connection-id should return 404 Not Found"
    );

    // Test missing connection ID returns 400
    let missing_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/message")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&json_rpc_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        missing_response.status(),
        StatusCode::BAD_REQUEST,
        "POST /mcp/message without connection-id should return 400 Bad Request"
    );
}

/// T018: Test call `get_ticker` via SSE returns valid ticker data within 2s
///
/// ## Acceptance Criteria
///
/// - SSE connection established successfully
/// - JSON-RPC call to `get_ticker` tool succeeds
/// - Response contains valid ticker data (symbol, price, timestamp)
/// - Response received within 2 seconds
/// - Tool behavior identical to stdio transport
#[tokio::test]
async fn test_get_ticker_via_sse_returns_valid_data() {
    let app = create_test_sse_router().await;

    // Establish SSE connection
    let handshake_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mcp/sse")
                .header("Accept", "text/event-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let connection_id = handshake_response
        .headers()
        .get("X-Connection-ID")
        .unwrap()
        .to_str()
        .unwrap();

    // Call get_ticker tool via JSON-RPC
    let get_ticker_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "get_ticker",
            "arguments": {
                "symbol": "BTCUSDT"
            }
        }
    });

    let start = std::time::Instant::now();

    let response = tokio::time::timeout(
        Duration::from_secs(2),
        app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/message")
                .header("Content-Type", "application/json")
                .header("X-Connection-ID", connection_id)
                .body(Body::from(
                    serde_json::to_string(&get_ticker_request).unwrap(),
                ))
                .unwrap(),
        ),
    )
    .await
    .expect("get_ticker should respond within 2 seconds")
    .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "get_ticker call should be accepted"
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "Response should arrive within 2 seconds, took {:?}",
        elapsed
    );

    // Parse SSE event stream to extract ticker data
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // SSE events have format: data: {...}\n\n
    let json_data = body_str
        .lines()
        .find(|line| line.starts_with("data: "))
        .map(|line| line.strip_prefix("data: ").unwrap())
        .expect("SSE stream should contain data event");

    let result: Value = serde_json::from_str(json_data).unwrap();

    // Validate ticker data structure
    assert_eq!(result["jsonrpc"], "2.0", "Should be JSON-RPC 2.0");
    assert_eq!(result["id"], 1, "Should match request ID");
    assert!(
        result["result"].is_object(),
        "Result should contain ticker data"
    );

    let ticker = &result["result"];
    assert_eq!(ticker["symbol"], "BTCUSDT", "Symbol should match request");
    assert!(
        ticker["price"].is_string() || ticker["price"].is_number(),
        "Price should be present"
    );
}

/// T019: Test 3 concurrent SSE connections all succeed and receive unique connection IDs
///
/// ## Acceptance Criteria
///
/// - 3 concurrent SSE handshakes all return 200 OK
/// - All 3 receive unique X-Connection-ID headers
/// - No connection IDs are duplicated
/// - All 3 connections remain active simultaneously
/// - Concurrent limit enforcement works (max 50 connections per SC-004)
#[tokio::test]
async fn test_concurrent_sse_connections_receive_unique_ids() {
    let app = create_test_sse_router().await;

    // Create 3 concurrent SSE connection requests
    let mut tasks = vec![];

    for i in 0..3 {
        let app_clone = app.clone();
        let task = tokio::spawn(async move {
            let response = app_clone
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri("/mcp/sse")
                        .header("Accept", "text/event-stream")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Concurrent connection {} should succeed",
                i
            );

            let connection_id = response
                .headers()
                .get("X-Connection-ID")
                .expect("Should receive connection ID")
                .to_str()
                .unwrap()
                .to_string();

            connection_id
        });

        tasks.push(task);
    }

    // Wait for all 3 connections to complete
    let connection_ids: Vec<String> = futures_util::future::join_all(tasks)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        .collect();

    // Verify we have 3 connection IDs
    assert_eq!(
        connection_ids.len(),
        3,
        "Should receive 3 connection IDs"
    );

    // Verify all IDs are unique (no duplicates)
    let unique_ids: std::collections::HashSet<_> = connection_ids.iter().collect();
    assert_eq!(
        unique_ids.len(),
        3,
        "All 3 connection IDs should be unique: {:?}",
        connection_ids
    );

    // Verify all IDs are valid UUID v4
    for (i, id) in connection_ids.iter().enumerate() {
        assert!(
            uuid::Uuid::parse_str(id).is_ok(),
            "Connection ID {} should be valid UUID: {}",
            i,
            id
        );
    }
}

/// Test max concurrent connections limit (SC-004)
///
/// ## Acceptance Criteria
///
/// - 50 concurrent connections succeed
/// - 51st connection returns HTTP 503 Service Unavailable
/// - Error message indicates max connections reached
#[tokio::test]
#[ignore] // Expensive test - run manually with: cargo test --features sse,orderbook_analytics -- --ignored
async fn test_max_concurrent_connections_enforced() {
    use mcp_binance_server::transport::sse::SessionManager;

    let session_manager = SessionManager::new();

    // Register 50 connections (should all succeed)
    let mut connection_ids = vec![];
    for i in 0..50 {
        let addr = format!("127.0.0.1:{}", 10000 + i)
            .parse()
            .unwrap();
        let conn_id = session_manager
            .register_connection(addr, Some(format!("test-agent-{}", i)))
            .await
            .expect("First 50 connections should succeed");
        connection_ids.push(conn_id);
    }

    assert_eq!(
        connection_ids.len(),
        50,
        "Should have 50 active connections"
    );

    // Try to register 51st connection (should fail)
    let addr = "127.0.0.1:60000".parse().unwrap();
    let result = session_manager
        .register_connection(addr, Some("test-agent-51".to_string()))
        .await;

    assert!(
        result.is_none(),
        "51st connection should be rejected when max limit reached"
    );

    // Verify connection count
    let count = session_manager.connection_count().await;
    assert_eq!(count, 50, "Should still have exactly 50 connections");
}
