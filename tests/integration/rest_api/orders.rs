//! Order Management API integration tests
//!
//! Tests for 5 authenticated order endpoints (sequential execution required):
//! - POST /api/v3/order - Place new order
//! - GET /api/v3/order - Query order status
//! - DELETE /api/v3/order - Cancel order
//! - GET /api/v3/openOrders - Query all open orders
//! - GET /api/v3/allOrders - Query all orders (open + closed)
//!
//! IMPORTANT: These tests must run sequentially (#[serial(orders)])
//! because they share Binance Testnet account state

use super::{
    assert_cors_headers, build_authenticated_url, setup_authenticated_client, wait_for_rate_limit,
};
use crate::common::{assertions, fixtures::SampleOrder, init_test_env};
use serial_test::serial;

/// T019: Test POST /api/v3/order endpoint
/// Places a new limit order and verifies response
#[tokio::test]
#[serial(orders)]
async fn test_place_order() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();
    let order = SampleOrder::limit_sell();

    let url = build_authenticated_url(
        &creds.base_url,
        "order",
        &[
            ("symbol", &order.symbol),
            ("side", &order.side),
            ("type", &order.order_type),
            ("quantity", &order.quantity),
            ("price", order.price.as_ref().unwrap()),
            ("timeInForce", order.time_in_force.as_ref().unwrap()),
        ],
        &creds.api_secret,
    );

    let response = client
        .post(&url)
        .send()
        .await
        .expect("Failed to send place order request");

    // Assert successful response (201 Created or 200 OK)
    let status = response.status();
    assert!(
        status == 200 || status == 201,
        "Expected 200/201 status, got {}",
        status
    );

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse and validate JSON schema
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse order response JSON");

    // Verify order response has required fields
    assertions::assert_has_fields(
        &json,
        &[
            "symbol",
            "orderId",
            "clientOrderId",
            "transactTime",
            "price",
            "origQty",
            "executedQty",
            "status",
            "type",
            "side",
        ],
    );

    // Verify data types
    assertions::assert_field_type(&json, "symbol", assertions::JsonType::String);
    assertions::assert_field_type(&json, "orderId", assertions::JsonType::Number);
    assertions::assert_field_type(&json, "status", assertions::JsonType::String);

    // Verify order details match request
    assert_eq!(json["symbol"].as_str().unwrap(), order.symbol);
    assert_eq!(json["side"].as_str().unwrap(), order.side);
    assert_eq!(json["type"].as_str().unwrap(), order.order_type);

    wait_for_rate_limit().await;
}

/// T020: Test GET /api/v3/order endpoint
/// Queries status of a specific order by orderId
#[tokio::test]
#[serial(orders)]
async fn test_query_order() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    // First, get an order ID by querying all orders
    let all_orders_url = build_authenticated_url(
        &creds.base_url,
        "allOrders",
        &[("symbol", "BTCUSDT"), ("limit", "1")],
        &creds.api_secret,
    );

    let all_orders_response = client
        .get(&all_orders_url)
        .send()
        .await
        .expect("Failed to get all orders");

    let orders: serde_json::Value = all_orders_response
        .json()
        .await
        .expect("Failed to parse all orders JSON");

    let orders_arr = orders.as_array().expect("Expected array of orders");

    // Skip if no orders exist
    if orders_arr.is_empty() {
        println!("No orders found, skipping query order test");
        return;
    }

    let first_order = &orders_arr[0];
    let order_id = first_order["orderId"]
        .as_i64()
        .expect("orderId should be number");

    // Now query specific order
    let url = build_authenticated_url(
        &creds.base_url,
        "order",
        &[("symbol", "BTCUSDT"), ("orderId", &order_id.to_string())],
        &creds.api_secret,
    );

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send query order request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse and validate JSON schema
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse order query JSON");

    // Verify order response has required fields
    assertions::assert_has_fields(&json, &["symbol", "orderId", "status", "price", "origQty"]);

    // Verify queried order ID matches
    assert_eq!(
        json["orderId"].as_i64().unwrap(),
        order_id,
        "Order ID should match query"
    );

    wait_for_rate_limit().await;
}

/// T021: Test DELETE /api/v3/order endpoint
/// Cancels an open order
#[tokio::test]
#[serial(orders)]
async fn test_cancel_order() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    // First, place a new limit order to cancel
    let order = SampleOrder::limit_sell();

    let place_url = build_authenticated_url(
        &creds.base_url,
        "order",
        &[
            ("symbol", &order.symbol),
            ("side", &order.side),
            ("type", &order.order_type),
            ("quantity", &order.quantity),
            ("price", order.price.as_ref().unwrap()),
            ("timeInForce", order.time_in_force.as_ref().unwrap()),
        ],
        &creds.api_secret,
    );

    let place_response = client
        .post(&place_url)
        .send()
        .await
        .expect("Failed to place order for cancellation");

    let place_json: serde_json::Value = place_response
        .json()
        .await
        .expect("Failed to parse place order JSON");

    let order_id = place_json["orderId"]
        .as_i64()
        .expect("orderId should be number");

    wait_for_rate_limit().await;

    // Now cancel the order
    let cancel_url = build_authenticated_url(
        &creds.base_url,
        "order",
        &[
            ("symbol", &order.symbol),
            ("orderId", &order_id.to_string()),
        ],
        &creds.api_secret,
    );

    let response = client
        .delete(&cancel_url)
        .send()
        .await
        .expect("Failed to send cancel order request");

    let status = response.status();

    // Assert CORS headers present (only for successful responses)
    if status == 200 {
        assert_cors_headers(&response);
    }

    // Parse JSON response
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse cancel order JSON");

    // Check if cancel succeeded or if order couldn't be canceled
    if status == 200 {
        // Successfully canceled
        assertions::assert_has_fields(&json, &["symbol", "orderId", "status"]);

        // Status should be CANCELED
        assert_eq!(
            json["status"].as_str().unwrap(),
            "CANCELED",
            "Order status should be CANCELED"
        );
    } else if status == 400 {
        // Order may have been filled or already canceled
        // This is acceptable in test environment
        println!("Order could not be canceled (status 400): {:?}", json);
        assert!(
            json.get("code").is_some() || json.get("msg").is_some(),
            "Error response should have code or msg"
        );
    } else {
        panic!("Unexpected status code {}: {:?}", status, json);
    }

    wait_for_rate_limit().await;
}

/// T022: Test GET /api/v3/openOrders endpoint
/// Queries all open orders for a symbol or all symbols
#[tokio::test]
#[serial(orders)]
async fn test_open_orders() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    let url = build_authenticated_url(
        &creds.base_url,
        "openOrders",
        &[("symbol", "BTCUSDT")],
        &creds.api_secret,
    );

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send open orders request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse JSON array
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse open orders JSON");

    let orders = json.as_array().expect("Expected array of orders");

    // Verify each order has required fields (if any orders exist)
    for order in orders {
        assertions::assert_has_fields(order, &["symbol", "orderId", "status", "price", "origQty"]);

        // All orders should have status != FILLED/CANCELED/REJECTED
        let status = order["status"].as_str().unwrap();
        assert!(
            status == "NEW" || status == "PARTIALLY_FILLED",
            "Open order should have NEW or PARTIALLY_FILLED status, got {}",
            status
        );
    }

    wait_for_rate_limit().await;
}

/// T023: Test GET /api/v3/allOrders endpoint
/// Queries all orders (open + closed) for a symbol
#[tokio::test]
#[serial(orders)]
async fn test_all_orders() {
    init_test_env();

    let (client, creds) = setup_authenticated_client();

    let url = build_authenticated_url(
        &creds.base_url,
        "allOrders",
        &[("symbol", "BTCUSDT"), ("limit", "10")],
        &creds.api_secret,
    );

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send all orders request");

    // Assert successful response
    assert_eq!(response.status(), 200, "Expected 200 OK status");

    // Assert CORS headers present
    assert_cors_headers(&response);

    // Parse JSON array
    let json: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse all orders JSON");

    let orders = json.as_array().expect("Expected array of orders");

    assert!(orders.len() <= 10, "Orders should not exceed limit of 10");

    // Verify each order has required fields
    for order in orders {
        assertions::assert_has_fields(
            order,
            &["symbol", "orderId", "status", "price", "origQty", "time"],
        );

        assertions::assert_field_type(order, "symbol", assertions::JsonType::String);
        assertions::assert_field_type(order, "orderId", assertions::JsonType::Number);
        assertions::assert_field_type(order, "status", assertions::JsonType::String);

        // Verify symbol matches request
        assert_eq!(
            order["symbol"].as_str().unwrap(),
            "BTCUSDT",
            "Order symbol should match request"
        );
    }

    wait_for_rate_limit().await;
}
