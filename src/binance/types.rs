//! Binance API Type Definitions
//!
//! Type definitions for Binance API responses and requests.
//! All types include validation and proper deserialization.

use serde::{Deserialize, Serialize};

/// Response from Binance /api/v3/time endpoint
///
/// Returns the current server time in milliseconds since Unix epoch.
/// Used for time synchronization and validating request signatures.
///
/// # Example Response
/// ```json
/// {
///   "serverTime": 1699564800000
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTimeResponse {
    /// Server time in milliseconds since Unix epoch
    ///
    /// Must be a positive i64 value. Typical range: 1600000000000 to 2000000000000
    pub server_time: i64,
}

impl ServerTimeResponse {
    /// Validates the server time is within reasonable bounds
    ///
    /// Returns true if server_time is positive (after Unix epoch).
    /// This prevents issues with negative timestamps or zero values.
    pub fn is_valid(&self) -> bool {
        self.server_time > 0
    }

    /// Returns the server time as milliseconds since Unix epoch
    pub fn time_ms(&self) -> i64 {
        self.server_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_time_deserialization() {
        let json = r#"{"serverTime": 1699564800000}"#;
        let response: ServerTimeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.server_time, 1699564800000);
        assert!(response.is_valid());
    }

    #[test]
    fn test_invalid_server_time() {
        let response = ServerTimeResponse { server_time: -1 };
        assert!(!response.is_valid());
    }

    #[test]
    fn test_zero_server_time() {
        let response = ServerTimeResponse { server_time: 0 };
        assert!(!response.is_valid());
    }
}
