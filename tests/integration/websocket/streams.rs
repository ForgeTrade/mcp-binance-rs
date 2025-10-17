//! WebSocket stream integration tests
//!
//! Tests for 3 WebSocket stream types:
//! - Ticker stream: Real-time 24hr ticker statistics
//! - Depth stream: Real-time order book updates
//! - User data stream: Account and order updates (authenticated)

use super::{
    build_stream_url, close_websocket, connect_websocket, get_test_ws_config, parse_json_message,
    receive_message_with_timeout,
};
use crate::common::{assertions, init_test_env};
use serial_test::serial;

/// T029: Test ticker WebSocket stream
/// Subscribes to btcusdt@ticker and verifies message structure
#[tokio::test]
async fn test_ticker_websocket_stream() {
    init_test_env();

    let (ws_base_url, _creds) = get_test_ws_config();
    let stream_url = build_stream_url(&ws_base_url, "btcusdt@ticker");

    // Connect to WebSocket
    let (mut ws_stream, response) = connect_websocket(&stream_url)
        .await
        .expect("Failed to connect to ticker stream");

    // Verify WebSocket upgrade response
    assert_eq!(
        response.status(),
        101,
        "Expected HTTP 101 Switching Protocols"
    );

    // Wait for first ticker message (30 second timeout)
    let msg = receive_message_with_timeout(&mut ws_stream, 30)
        .await
        .expect("Failed to receive ticker message")
        .expect("Stream closed unexpectedly");

    // Parse JSON message
    let json = parse_json_message(&msg).expect("Failed to parse ticker JSON");

    // Verify ticker message structure
    assertions::assert_has_fields(&json, &["e", "E", "s", "p", "P", "c", "o", "h", "l", "v"]);

    // Verify event type is ticker
    assert_eq!(
        json["e"].as_str().unwrap(),
        "24hrTicker",
        "Event type should be 24hrTicker"
    );

    // Verify symbol is BTCUSDT
    assert_eq!(
        json["s"].as_str().unwrap(),
        "BTCUSDT",
        "Symbol should be BTCUSDT"
    );

    // Verify price fields are strings (Binance format)
    assertions::assert_field_type(&json, "c", assertions::JsonType::String); // Current price
    assertions::assert_field_type(&json, "p", assertions::JsonType::String); // Price change
    assertions::assert_field_type(&json, "P", assertions::JsonType::String); // Price change percent

    // Close connection
    close_websocket(&mut ws_stream)
        .await
        .expect("Failed to close WebSocket");
}

/// T030: Test depth WebSocket stream
/// Subscribes to btcusdt@depth and verifies order book updates
#[tokio::test]
async fn test_depth_websocket_stream() {
    init_test_env();

    let (ws_base_url, _creds) = get_test_ws_config();
    let stream_url = build_stream_url(&ws_base_url, "btcusdt@depth");

    // Connect to WebSocket
    let (mut ws_stream, response) = connect_websocket(&stream_url)
        .await
        .expect("Failed to connect to depth stream");

    // Verify WebSocket upgrade response
    assert_eq!(
        response.status(),
        101,
        "Expected HTTP 101 Switching Protocols"
    );

    // Wait for first depth message (30 second timeout)
    let msg = receive_message_with_timeout(&mut ws_stream, 30)
        .await
        .expect("Failed to receive depth message")
        .expect("Stream closed unexpectedly");

    // Parse JSON message
    let json = parse_json_message(&msg).expect("Failed to parse depth JSON");

    // Verify depth message structure
    assertions::assert_has_fields(&json, &["e", "E", "s", "U", "u", "b", "a"]);

    // Verify event type is depthUpdate
    assert_eq!(
        json["e"].as_str().unwrap(),
        "depthUpdate",
        "Event type should be depthUpdate"
    );

    // Verify symbol is BTCUSDT
    assert_eq!(
        json["s"].as_str().unwrap(),
        "BTCUSDT",
        "Symbol should be BTCUSDT"
    );

    // Verify bids and asks are arrays
    assertions::assert_field_type(&json, "b", assertions::JsonType::Array); // Bids
    assertions::assert_field_type(&json, "a", assertions::JsonType::Array); // Asks

    // Verify update IDs are numbers
    assertions::assert_field_type(&json, "U", assertions::JsonType::Number); // First update ID
    assertions::assert_field_type(&json, "u", assertions::JsonType::Number); // Final update ID

    // Close connection
    close_websocket(&mut ws_stream)
        .await
        .expect("Failed to close WebSocket");
}

/// T031: Test user data stream subscription
/// Tests starting a user data stream and receiving listenKey
#[tokio::test]
#[serial(user_stream)]
async fn test_user_data_stream_subscription() {
    init_test_env();

    let (_ws_base_url, creds) = get_test_ws_config();

    // First, start user data stream via REST API to get listenKey
    let client = reqwest::Client::new();
    let start_url = format!("{}/api/v3/userDataStream", creds.base_url);

    let response = client
        .post(&start_url)
        .header("X-MBX-APIKEY", &creds.api_key)
        .send()
        .await
        .expect("Failed to start user data stream");

    assert_eq!(response.status(), 200, "Expected 200 OK for stream start");

    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse stream start response");

    // Verify listenKey is present
    assertions::assert_has_fields(&json, &["listenKey"]);
    let listen_key = json["listenKey"]
        .as_str()
        .expect("listenKey should be string");

    assert!(!listen_key.is_empty(), "listenKey should not be empty");
    assert!(
        listen_key.len() >= 40,
        "listenKey should be at least 40 characters"
    );

    // Note: Actual WebSocket connection to user data stream tested in T032
}

/// T032: Test user data stream connection
/// Connects to user data stream with listenKey and verifies connection
#[tokio::test]
#[serial(user_stream)]
async fn test_user_data_stream_connection() {
    init_test_env();

    let (ws_base_url, creds) = get_test_ws_config();

    // Start user data stream to get listenKey
    let client = reqwest::Client::new();
    let start_url = format!("{}/api/v3/userDataStream", creds.base_url);

    let response = client
        .post(&start_url)
        .header("X-MBX-APIKEY", &creds.api_key)
        .send()
        .await
        .expect("Failed to start user data stream");

    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse stream start response");

    let listen_key = json["listenKey"]
        .as_str()
        .expect("listenKey should be string");

    // Connect to WebSocket with listenKey
    let stream_url = build_stream_url(&ws_base_url, listen_key);

    let (mut ws_stream, response) = connect_websocket(&stream_url)
        .await
        .expect("Failed to connect to user data stream");

    // Verify WebSocket upgrade response
    assert_eq!(
        response.status(),
        101,
        "Expected HTTP 101 Switching Protocols"
    );

    // Wait for first message or timeout (user data stream may not send immediate messages)
    // This is just to verify the connection is stable
    let result = receive_message_with_timeout(&mut ws_stream, 10).await;

    // Connection success is verified by successful WebSocket handshake
    // Messages may or may not arrive depending on account activity
    match result {
        Ok(Some(msg)) => {
            // Try to parse as JSON, but Ping/Pong messages are also acceptable
            match parse_json_message(&msg) {
                Ok(_) => println!("Received user data message"),
                Err(e) => {
                    // Ping/Pong or other non-JSON messages are normal
                    println!("Received non-JSON message (Ping/Pong): {}", e);
                }
            }
        }
        Ok(None) => {
            println!("Stream closed by server");
        }
        Err(e) => {
            // Timeout is acceptable - no account activity means no messages
            println!(
                "No messages received (expected if no account activity): {}",
                e
            );
        }
    }

    // Close connection
    close_websocket(&mut ws_stream)
        .await
        .expect("Failed to close WebSocket");

    // Clean up: Close user data stream
    let close_url = format!(
        "{}/api/v3/userDataStream?listenKey={}",
        creds.base_url, listen_key
    );

    let _close_response = client
        .delete(&close_url)
        .header("X-MBX-APIKEY", &creds.api_key)
        .send()
        .await
        .expect("Failed to close user data stream");
}

/// T033: Test WebSocket reconnection handling
/// Tests connection stability and reconnection behavior
#[tokio::test]
async fn test_websocket_reconnection() {
    init_test_env();

    let (ws_base_url, _creds) = get_test_ws_config();
    let stream_url = build_stream_url(&ws_base_url, "btcusdt@ticker");

    // Connect to WebSocket
    let (mut ws_stream, _response) = connect_websocket(&stream_url)
        .await
        .expect("Failed to connect to ticker stream");

    // Receive first message to confirm connection works
    let _msg1 = receive_message_with_timeout(&mut ws_stream, 30)
        .await
        .expect("Failed to receive first message")
        .expect("Stream closed unexpectedly");

    // Close connection
    close_websocket(&mut ws_stream)
        .await
        .expect("Failed to close WebSocket");

    // Wait a moment before reconnecting
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Reconnect to same stream
    let (mut ws_stream2, response2) = connect_websocket(&stream_url)
        .await
        .expect("Failed to reconnect to ticker stream");

    // Verify reconnection successful
    assert_eq!(response2.status(), 101, "Expected HTTP 101 on reconnection");

    // Receive message on new connection
    let msg2 = receive_message_with_timeout(&mut ws_stream2, 30)
        .await
        .expect("Failed to receive message after reconnection")
        .expect("Stream closed unexpectedly");

    // Verify message is valid
    let _json = parse_json_message(&msg2).expect("Failed to parse ticker JSON");

    // Close connection
    close_websocket(&mut ws_stream2)
        .await
        .expect("Failed to close WebSocket");
}

/// T034: Test multiple stream subscriptions
/// Tests subscribing to multiple streams simultaneously
#[tokio::test]
async fn test_multiple_stream_subscriptions() {
    init_test_env();

    let (ws_base_url, _creds) = get_test_ws_config();

    // Create combined stream URL for multiple streams
    // Format: /stream?streams=btcusdt@ticker/ethusdt@ticker
    let combined_stream = format!(
        "{}/stream?streams=btcusdt@ticker/ethusdt@ticker",
        ws_base_url
    );

    // Connect to combined stream
    let (mut ws_stream, response) = connect_websocket(&combined_stream)
        .await
        .expect("Failed to connect to combined stream");

    // Verify WebSocket upgrade response
    assert_eq!(
        response.status(),
        101,
        "Expected HTTP 101 Switching Protocols"
    );

    // Collect messages from both streams
    let mut btc_received = false;
    let mut eth_received = false;
    let mut attempts = 0;
    let max_attempts = 10;

    while (!btc_received || !eth_received) && attempts < max_attempts {
        let msg = receive_message_with_timeout(&mut ws_stream, 30)
            .await
            .expect("Failed to receive message")
            .expect("Stream closed unexpectedly");

        let json = parse_json_message(&msg).expect("Failed to parse JSON");

        // Combined streams wrap messages in {"stream": "...", "data": {...}}
        if json.get("stream").is_some() {
            let stream_name = json["stream"].as_str().unwrap();
            let data = &json["data"];

            if stream_name == "btcusdt@ticker" {
                assert_eq!(data["s"].as_str().unwrap(), "BTCUSDT");
                btc_received = true;
            } else if stream_name == "ethusdt@ticker" {
                assert_eq!(data["s"].as_str().unwrap(), "ETHUSDT");
                eth_received = true;
            }
        } else {
            // Single stream format (fallback)
            let symbol = json["s"].as_str().unwrap_or("");
            if symbol == "BTCUSDT" {
                btc_received = true;
            } else if symbol == "ETHUSDT" {
                eth_received = true;
            }
        }

        attempts += 1;
    }

    // Verify both streams sent messages
    assert!(
        btc_received,
        "Should receive at least one BTCUSDT ticker message"
    );
    assert!(
        eth_received,
        "Should receive at least one ETHUSDT ticker message"
    );

    // Close connection
    close_websocket(&mut ws_stream)
        .await
        .expect("Failed to close WebSocket");
}
