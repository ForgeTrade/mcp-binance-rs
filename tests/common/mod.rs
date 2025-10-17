//! Common test utilities and fixtures for integration tests
//!
//! This module provides shared test infrastructure including:
//! - Test credentials and authentication fixtures
//! - Binance Testnet client configuration
//! - Custom assertion helpers for JSON schema validation
//! - Environment variable loading and initialization

use std::sync::Once;

pub mod assertions;
pub mod binance_client;
pub mod fixtures;

static INIT: Once = Once::new();

/// Initialize test environment once per test run
/// Loads .env.test file and sets up logging
pub fn init_test_env() {
    INIT.call_once(|| {
        // Load .env.test file if it exists
        dotenv::from_filename(".env.test").ok();

        // Initialize logging for tests
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
            .try_init()
            .ok();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_test_env() {
        init_test_env();
        // Should not panic on multiple calls
        init_test_env();
    }
}
