//! Integration tests for Streamable HTTP transport (Feature 010)
//!
//! These tests verify Streamable HTTP protocol (MCP March 2025 spec):
//! - POST /mcp with initialize method creates session
//! - Mcp-Session-Id header returned and validated
//! - Tool calls work over Streamable HTTP
//!
//! ## Test Coverage
//!
//! - T016: POST /mcp initialize creates session and returns Mcp-Session-Id header
//! - T017: POST /mcp with valid Mcp-Session-Id executes tools/list
//! - T018: Call `get_ticker` via Streamable HTTP returns valid ticker data within 2s
//! - T019: 3 concurrent sessions all succeed and receive unique Mcp-Session-Id values
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
    use mcp_binance_server::transport::sse::{message_post, SessionManager, SseState};

    let session_manager = SessionManager::new();
    let mcp_server = BinanceServer::new();
    let state = SseState::new(session_manager, mcp_server);

    // Create router with Streamable HTTP endpoint (March 2025 spec)
    axum::Router::new()
        .route("/mcp", axum::routing::post(message_post))
        .with_state(state)
}

/// T016: Test POST /mcp initialize creates session and returns Mcp-Session-Id header
///
/// ## Acceptance Criteria (Streamable HTTP spec)
///
/// - POST /mcp with initialize method returns 200 OK
/// - Response headers include Mcp-Session-Id
/// - Session ID is valid UUID v4 format
/// - Response is valid JSON-RPC 2.0 initialize response
#[tokio::test]
async fn test_initialize_returns_session_id() {
    let app = create_test_sse_router().await;

    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0"
            }
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&initialize_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert HTTP 200 OK
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Initialize should return 200 OK"
    );

    // Assert Mcp-Session-Id header exists
    let session_id = response
        .headers()
        .get("Mcp-Session-Id")
        .expect("Response should include Mcp-Session-Id header");

    let session_id_str = session_id.to_str().unwrap();

    // Assert session ID is valid UUID v4 format
    assert!(
        uuid::Uuid::parse_str(session_id_str).is_ok(),
        "Session ID should be valid UUID v4: {}",
        session_id_str
    );

    // Parse and validate JSON-RPC response
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(result["jsonrpc"], "2.0", "Should be JSON-RPC 2.0");
    assert_eq!(result["id"], 1, "Should match request ID");
    assert!(
        result["result"]["serverInfo"].is_object(),
        "Should contain server info"
    );
}

/// T017: Test POST /mcp with valid Mcp-Session-Id executes tools/list
///
/// ## Acceptance Criteria (Streamable HTTP spec)
///
/// - POST /mcp with valid Mcp-Session-Id returns 200 OK
/// - Invalid Mcp-Session-Id returns 404 Not Found
/// - Missing Mcp-Session-Id returns 400 Bad Request
/// - Request body must be valid JSON-RPC 2.0
#[tokio::test]
async fn test_post_mcp_with_valid_session_id() {
    let app = create_test_sse_router().await;

    // First, initialize to get session ID
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    });

    let init_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&initialize_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let session_id = init_response
        .headers()
        .get("Mcp-Session-Id")
        .expect("Initialize should return session ID")
        .to_str()
        .unwrap();

    // Test valid session ID with tools/list request
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .header("Mcp-Session-Id", session_id)
                .body(Body::from(serde_json::to_string(&tools_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "POST /mcp with valid session should return 200 OK"
    );

    // Test invalid session ID returns 404
    let invalid_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .header("Mcp-Session-Id", "invalid-uuid")
                .body(Body::from(serde_json::to_string(&tools_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        invalid_response.status(),
        StatusCode::NOT_FOUND,
        "POST /mcp with invalid session should return 404 Not Found"
    );

    // Test missing session ID returns 400
    let missing_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&tools_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        missing_response.status(),
        StatusCode::BAD_REQUEST,
        "POST /mcp without session should return 400 Bad Request"
    );
}

/// T018: Test call `get_ticker` via Streamable HTTP returns valid ticker data within 2s
///
/// ## Acceptance Criteria (Streamable HTTP spec)
///
/// - POST /mcp with initialize method creates session
/// - JSON-RPC call to `get_ticker` tool succeeds
/// - Response contains valid ticker data (symbol, price, timestamp)
/// - Response received within 2 seconds
/// - Tool behavior identical to stdio transport
#[tokio::test]
async fn test_get_ticker_via_streamable_http_returns_valid_data() {
    let app = create_test_sse_router().await;

    // Initialize session
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    });

    let init_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&initialize_request).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let session_id = init_response
        .headers()
        .get("Mcp-Session-Id")
        .expect("Initialize should return session ID")
        .to_str()
        .unwrap();

    // Call get_ticker tool via JSON-RPC
    let get_ticker_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
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
                .uri("/mcp")
                .header("Content-Type", "application/json")
                .header("Mcp-Session-Id", session_id)
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
        StatusCode::OK,
        "get_ticker call should return 200 OK"
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "Response should arrive within 2 seconds, took {:?}",
        elapsed
    );

    // Parse JSON-RPC response
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Validate ticker data structure
    assert_eq!(result["jsonrpc"], "2.0", "Should be JSON-RPC 2.0");
    assert_eq!(result["id"], 2, "Should match request ID");
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

/// T019: Test 3 concurrent sessions all succeed and receive unique Mcp-Session-Id values
///
/// ## Acceptance Criteria (Streamable HTTP spec)
///
/// - 3 concurrent POST /mcp initialize calls all return 200 OK
/// - All 3 receive unique Mcp-Session-Id headers
/// - No session IDs are duplicated
/// - All 3 sessions remain active simultaneously
/// - Concurrent limit enforcement works (max 50 sessions per SC-004)
#[tokio::test]
async fn test_concurrent_sessions_receive_unique_ids() {
    let app = create_test_sse_router().await;

    // Create 3 concurrent initialize requests
    let mut tasks = vec![];

    for i in 0..3 {
        let app_clone = app.clone();
        let task = tokio::spawn(async move {
            let initialize_request = serde_json::json!({
                "jsonrpc": "2.0",
                "id": i + 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": format!("test-client-{}", i),
                        "version": "1.0"
                    }
                }
            });

            let response = app_clone
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/mcp")
                        .header("Content-Type", "application/json")
                        .body(Body::from(
                            serde_json::to_string(&initialize_request).unwrap(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Concurrent session {} should succeed",
                i
            );

            let session_id = response
                .headers()
                .get("Mcp-Session-Id")
                .expect("Should receive session ID")
                .to_str()
                .unwrap()
                .to_string();

            session_id
        });

        tasks.push(task);
    }

    // Wait for all 3 sessions to complete
    let session_ids: Vec<String> = futures_util::future::join_all(tasks)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        .collect();

    // Verify we have 3 session IDs
    assert_eq!(session_ids.len(), 3, "Should receive 3 session IDs");

    // Verify all IDs are unique (no duplicates)
    let unique_ids: std::collections::HashSet<_> = session_ids.iter().collect();
    assert_eq!(
        unique_ids.len(),
        3,
        "All 3 session IDs should be unique: {:?}",
        session_ids
    );

    // Verify all IDs are valid UUID v4
    for (i, id) in session_ids.iter().enumerate() {
        assert!(
            uuid::Uuid::parse_str(id).is_ok(),
            "Session ID {} should be valid UUID: {}",
            i,
            id
        );
    }
}

/// Test max concurrent sessions limit (SC-004)
///
/// ## Acceptance Criteria (Streamable HTTP spec)
///
/// - 50 concurrent sessions succeed
/// - 51st session registration fails
/// - SessionManager enforces max session limit (50)
#[tokio::test]
#[ignore] // Expensive test - run manually with: cargo test --features sse,orderbook_analytics -- --ignored
async fn test_max_concurrent_connections_enforced() {
    use mcp_binance_server::transport::sse::SessionManager;

    let session_manager = SessionManager::new();

    // Register 50 connections (should all succeed)
    let mut connection_ids = vec![];
    for i in 0..50 {
        let addr = format!("127.0.0.1:{}", 10000 + i).parse().unwrap();
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
