//! Resource URI Handling
//!
//! This module defines the resource URI parser and category types for MCP resources
//! including market data, account balances, and order information.

/// Resource category types for URI parsing (T025)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceCategory {
    /// Market data resources (e.g., binance://market/btcusdt)
    Market,
    /// Account balance resources (e.g., binance://account/balances)
    Account,
    /// Order information resources (e.g., binance://orders/open)
    Orders,
}

/// Parsed resource URI structure (T026)
#[derive(Debug, Clone)]
pub struct ResourceUri {
    /// URI scheme (always "binance")
    pub scheme: String,
    /// Resource category (market/account/orders)
    pub category: ResourceCategory,
    /// Optional resource identifier (e.g., "btcusdt", "balances", "open")
    pub identifier: Option<String>,
}

impl ResourceUri {
    /// Parse a resource URI string (T027)
    ///
    /// Expected format: `binance://{category}/{identifier}`
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_binance_server::server::resources::{ResourceUri, ResourceCategory};
    ///
    /// let uri = ResourceUri::parse("binance://market/btcusdt").unwrap();
    /// assert_eq!(uri.category, ResourceCategory::Market);
    /// assert_eq!(uri.identifier, Some("btcusdt".to_string()));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error string if:
    /// - URI doesn't start with "binance://"
    /// - Category is not one of: market, account, orders
    /// - URI format is invalid
    pub fn parse(uri: &str) -> Result<Self, String> {
        // Split by "://"
        let parts: Vec<&str> = uri.split("://").collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid URI format. Expected 'binance://{{category}}/{{identifier}}', got '{}'",
                uri
            ));
        }

        // Verify scheme
        if parts[0] != "binance" {
            return Err(format!(
                "Invalid scheme. Expected 'binance://', got '{}://'",
                parts[0]
            ));
        }

        // Parse path (category/identifier)
        let path_parts: Vec<&str> = parts[1].split('/').collect();
        if path_parts.is_empty() {
            return Err("Missing resource category".to_string());
        }

        // Parse category
        let category = match path_parts[0] {
            "market" => ResourceCategory::Market,
            "account" => ResourceCategory::Account,
            "orders" => ResourceCategory::Orders,
            other => {
                return Err(format!(
                    "Unknown category: '{}'. Valid categories: market, account, orders",
                    other
                ));
            }
        };

        // Parse optional identifier
        let identifier = if path_parts.len() > 1 {
            Some(path_parts[1].to_string())
        } else {
            None
        };

        Ok(ResourceUri {
            scheme: "binance".to_string(),
            category,
            identifier,
        })
    }
}
