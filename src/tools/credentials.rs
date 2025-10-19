//! Credential Management Tools (SSE feature only)
//!
//! Provides MCP tools for session-scoped Binance API credential management:
//! - `configure_credentials`: Store API keys for session
//! - `get_credentials_status`: View credential configuration status
//! - `revoke_credentials`: Clear credentials from session
//!
//! Implements Feature 011: Mainnet Support with Secure API Key Authentication
//!
//! **Note**: This module requires the `sse` feature flag to be enabled.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::CredentialError;

/// API key validation regex: exactly 64 alphanumeric characters
///
/// Uses Lazy static compilation for performance (FR-010, SC-007: <10ms validation).
static API_KEY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9]{64}$").expect("API key regex compilation failed"));

/// API secret validation regex: exactly 64 alphanumeric characters
///
/// Uses Lazy static compilation for performance (FR-010, SC-007: <10ms validation).
static API_SECRET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9]{64}$").expect("API secret regex compilation failed"));

/// Validates API key format (T012)
///
/// Checks that API key matches the expected format: exactly 64 alphanumeric characters.
/// This is synchronous format validation only - API validation occurs asynchronously
/// on first authenticated tool call.
///
/// # Arguments
///
/// * `api_key` - API key string to validate
///
/// # Returns
///
/// * `Ok(())` if format is valid
/// * `Err(CredentialError::InvalidApiKeyFormat)` if format is invalid
///
/// # Performance
///
/// < 10ms synchronous validation (FR-010, SC-007)
///
/// # Examples
///
/// ```
/// use mcp_binance_server::tools::credentials::validate_api_key;
///
/// // Valid: 64 alphanumeric characters
/// assert!(validate_api_key("A".repeat(64).as_str()).is_ok());
///
/// // Invalid: too short
/// assert!(validate_api_key("short").is_err());
///
/// // Invalid: contains special characters
/// assert!(validate_api_key(&format!("{}!", "A".repeat(63))).is_err());
/// ```
pub fn validate_api_key(api_key: &str) -> Result<(), CredentialError> {
    if API_KEY_REGEX.is_match(api_key) {
        Ok(())
    } else {
        Err(CredentialError::InvalidApiKeyFormat(
            "API key must be exactly 64 alphanumeric characters".to_string(),
        ))
    }
}

/// Validates API secret format (T013)
///
/// Checks that API secret matches the expected format: exactly 64 alphanumeric characters.
/// This is synchronous format validation only - API validation occurs asynchronously
/// on first authenticated tool call.
///
/// # Arguments
///
/// * `api_secret` - API secret string to validate
///
/// # Returns
///
/// * `Ok(())` if format is valid
/// * `Err(CredentialError::InvalidApiSecretFormat)` if format is invalid
///
/// # Performance
///
/// < 10ms synchronous validation (FR-010, SC-007)
///
/// # Examples
///
/// ```
/// use mcp_binance_server::tools::credentials::validate_api_secret;
///
/// // Valid: 64 alphanumeric characters
/// assert!(validate_api_secret(&"B".repeat(64)).is_ok());
///
/// // Invalid: too long
/// assert!(validate_api_secret(&"C".repeat(65)).is_err());
///
/// // Invalid: contains whitespace
/// assert!(validate_api_secret(&format!("{} ", "D".repeat(63))).is_err());
/// ```
pub fn validate_api_secret(api_secret: &str) -> Result<(), CredentialError> {
    if API_SECRET_REGEX.is_match(api_secret) {
        Ok(())
    } else {
        Err(CredentialError::InvalidApiSecretFormat(
            "API secret must be exactly 64 alphanumeric characters".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key_valid() {
        // Valid: exactly 64 alphanumeric
        let valid_key = "A".repeat(64);
        assert!(validate_api_key(&valid_key).is_ok());

        // Valid: mixed case
        let mixed_case = "AbCd".repeat(16); // 64 chars
        assert!(validate_api_key(&mixed_case).is_ok());

        // Valid: with numbers
        let with_numbers = "a1B2c3".repeat(10) + &"D4e5".to_string(); // 64 chars
        assert!(validate_api_key(&with_numbers).is_ok());
    }

    #[test]
    fn test_validate_api_key_invalid_length() {
        // Too short
        assert!(validate_api_key("short").is_err());

        // Too long
        let too_long = "X".repeat(65);
        assert!(validate_api_key(&too_long).is_err());

        // Empty
        assert!(validate_api_key("").is_err());
    }

    #[test]
    fn test_validate_api_key_invalid_characters() {
        // Special characters
        let with_special = format!("{}!", "A".repeat(63));
        assert!(validate_api_key(&with_special).is_err());

        // Whitespace
        let with_space = format!("{} ", "B".repeat(63));
        assert!(validate_api_key(&with_space).is_err());

        // Hyphen
        let with_hyphen = format!("{}-", "C".repeat(63));
        assert!(validate_api_key(&with_hyphen).is_err());
    }

    #[test]
    fn test_validate_api_secret_valid() {
        // Valid: exactly 64 alphanumeric
        let valid_secret = "X".repeat(64);
        assert!(validate_api_secret(&valid_secret).is_ok());
    }

    #[test]
    fn test_validate_api_secret_invalid() {
        // Too short
        assert!(validate_api_secret("secret").is_err());

        // Contains special characters
        let with_special = format!("{}@", "Y".repeat(63));
        assert!(validate_api_secret(&with_special).is_err());
    }
}
