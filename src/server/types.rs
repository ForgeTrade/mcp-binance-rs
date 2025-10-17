//! Prompt Parameter Types
//!
//! This module defines the parameter types for MCP prompts including trading analysis
//! and portfolio risk assessment.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Arguments for trading_analysis prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TradingAnalysisArgs {
    /// Trading pair symbol (e.g., BTCUSDT, ETHUSDT)
    #[schemars(description = "Trading pair symbol (e.g., BTCUSDT, ETHUSDT)")]
    pub symbol: String,

    /// Optional trading strategy preference
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Trading strategy preference: aggressive, balanced, or conservative")]
    pub strategy: Option<TradingStrategy>,

    /// Optional risk tolerance level
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Risk tolerance level: low, medium, or high")]
    pub risk_tolerance: Option<RiskTolerance>,
}

/// Trading strategy preference
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TradingStrategy {
    /// High-frequency, short-term trades
    Aggressive,
    /// Mixed approach balancing risk and reward
    Balanced,
    /// Low-risk, long-term holds
    Conservative,
}

/// Risk tolerance level
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RiskTolerance {
    /// Risk-averse, prefer stable assets
    Low,
    /// Moderate risk acceptable
    Medium,
    /// High-risk, high-reward tolerance
    High,
}

/// Arguments for portfolio_risk prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioRiskArgs {
    // Empty struct - no parameters required
    // Account info is derived from API credentials
}
