//! Custom assertion helpers for JSON schema validation
//!
//! Provides utilities to verify API responses match expected structure:
//! - Required fields are present
//! - Data types are correct
//! - Nested objects have proper structure

use serde_json::Value;

/// Assert that a JSON value contains all required fields
pub fn assert_has_fields(json: &Value, required_fields: &[&str]) {
    let obj = json.as_object().expect("Expected JSON object");

    for field in required_fields {
        assert!(
            obj.contains_key(*field),
            "Missing required field: {}",
            field
        );
    }
}

/// Assert that a JSON field has the expected type
pub fn assert_field_type(json: &Value, field: &str, expected_type: JsonType) {
    let obj = json.as_object().expect("Expected JSON object");
    let value = obj
        .get(field)
        .unwrap_or_else(|| panic!("Field {} not found", field));

    match expected_type {
        JsonType::String => assert!(value.is_string(), "Field {} is not a string", field),
        JsonType::Number => assert!(value.is_number(), "Field {} is not a number", field),
        JsonType::Boolean => assert!(value.is_boolean(), "Field {} is not a boolean", field),
        JsonType::Array => assert!(value.is_array(), "Field {} is not an array", field),
        JsonType::Object => assert!(value.is_object(), "Field {} is not an object", field),
        JsonType::Null => assert!(value.is_null(), "Field {} is not null", field),
    }
}

/// Supported JSON types for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

/// Assert that a ticker response has the expected schema
/// Used for 24hr ticker endpoint validation
pub fn assert_ticker_schema(json: &Value) {
    assert_has_fields(
        json,
        &[
            "symbol",
            "lastPrice",
            "volume",
            "priceChange",
            "priceChangePercent",
        ],
    );

    assert_field_type(json, "symbol", JsonType::String);
    assert_field_type(json, "lastPrice", JsonType::String);
    assert_field_type(json, "volume", JsonType::String);
}

/// Assert that an order book depth response has the expected schema
pub fn assert_depth_schema(json: &Value) {
    assert_has_fields(json, &["lastUpdateId", "bids", "asks"]);

    assert_field_type(json, "lastUpdateId", JsonType::Number);
    assert_field_type(json, "bids", JsonType::Array);
    assert_field_type(json, "asks", JsonType::Array);

    // Verify bid/ask arrays contain price/quantity pairs
    let bids = json["bids"].as_array().expect("bids should be array");
    if let Some(first_bid) = bids.first() {
        assert!(first_bid.is_array(), "Each bid should be an array");
        assert_eq!(
            first_bid.as_array().unwrap().len(),
            2,
            "Bid should have [price, quantity]"
        );
    }
}

/// Assert that account info response has the expected schema
pub fn assert_account_schema(json: &Value) {
    assert_has_fields(
        json,
        &[
            "makerCommission",
            "takerCommission",
            "canTrade",
            "canWithdraw",
            "canDeposit",
            "balances",
        ],
    );

    assert_field_type(json, "canTrade", JsonType::Boolean);
    assert_field_type(json, "balances", JsonType::Array);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_assert_has_fields() {
        let data = json!({
            "symbol": "BTCUSDT",
            "price": "50000.00"
        });

        assert_has_fields(&data, &["symbol", "price"]);
    }

    #[test]
    #[should_panic(expected = "Missing required field")]
    fn test_assert_has_fields_missing() {
        let data = json!({
            "symbol": "BTCUSDT"
        });

        assert_has_fields(&data, &["symbol", "price"]);
    }

    #[test]
    fn test_assert_field_type() {
        let data = json!({
            "name": "test",
            "count": 42,
            "active": true,
            "tags": [],
            "meta": {}
        });

        assert_field_type(&data, "name", JsonType::String);
        assert_field_type(&data, "count", JsonType::Number);
        assert_field_type(&data, "active", JsonType::Boolean);
        assert_field_type(&data, "tags", JsonType::Array);
        assert_field_type(&data, "meta", JsonType::Object);
    }

    #[test]
    fn test_assert_ticker_schema() {
        let ticker = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "50000.00",
            "volume": "1000.00",
            "priceChange": "500.00",
            "priceChangePercent": "1.00"
        });

        assert_ticker_schema(&ticker);
    }

    #[test]
    fn test_assert_depth_schema() {
        let depth = json!({
            "lastUpdateId": 123456,
            "bids": [["50000.00", "1.5"]],
            "asks": [["50100.00", "2.0"]]
        });

        assert_depth_schema(&depth);
    }
}
