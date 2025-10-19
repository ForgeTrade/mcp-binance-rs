/// Core domain types for Binance MCP Server
///
/// This module contains shared type definitions used across the server,
/// including environment configuration and credential management types.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Binance trading environment selection
///
/// Determines which Binance API endpoint to use for authenticated requests:
/// - Testnet: `https://testnet.binance.vision` (for testing with fake money)
/// - Mainnet: `https://api.binance.com` (for real trading with real money)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Testnet,
    Mainnet,
}

impl Environment {
    /// Returns the base URL for the Binance API based on the environment
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_binance_server::types::Environment;
    ///
    /// assert_eq!(Environment::Testnet.base_url(), "https://testnet.binance.vision");
    /// assert_eq!(Environment::Mainnet.base_url(), "https://api.binance.com");
    /// ```
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::Testnet => "https://testnet.binance.vision",
            Self::Mainnet => "https://api.binance.com",
        }
    }
}

impl FromStr for Environment {
    type Err = String;

    /// Parse environment string (case-insensitive: "testnet" or "mainnet")
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "testnet" => Ok(Self::Testnet),
            "mainnet" => Ok(Self::Mainnet),
            _ => Err(format!(
                "Invalid environment '{}'. Must be 'testnet' or 'mainnet'",
                s
            )),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Testnet => write!(f, "testnet"),
            Self::Mainnet => write!(f, "mainnet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_base_url() {
        assert_eq!(
            Environment::Testnet.base_url(),
            "https://testnet.binance.vision"
        );
        assert_eq!(Environment::Mainnet.base_url(), "https://api.binance.com");
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!(
            Environment::from_str("testnet").unwrap(),
            Environment::Testnet
        );
        assert_eq!(
            Environment::from_str("TESTNET").unwrap(),
            Environment::Testnet
        );
        assert_eq!(
            Environment::from_str("mainnet").unwrap(),
            Environment::Mainnet
        );
        assert_eq!(
            Environment::from_str("MAINNET").unwrap(),
            Environment::Mainnet
        );
        assert!(Environment::from_str("production").is_err());
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Testnet.to_string(), "testnet");
        assert_eq!(Environment::Mainnet.to_string(), "mainnet");
    }

    #[test]
    fn test_environment_serde() {
        let testnet = Environment::Testnet;
        let json = serde_json::to_string(&testnet).unwrap();
        assert_eq!(json, "\"testnet\"");

        let mainnet: Environment = serde_json::from_str("\"mainnet\"").unwrap();
        assert_eq!(mainnet, Environment::Mainnet);
    }
}
