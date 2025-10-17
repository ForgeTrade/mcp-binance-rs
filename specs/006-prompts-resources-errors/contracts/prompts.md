# Prompts API Contract

**Feature**: [../spec.md](../spec.md)
**Requirements**: FR-001 to FR-007
**Created**: 2025-10-17

## Contract Overview

This document defines the MCP Prompts API contract for the Binance MCP Server, including prompt registration, invocation, and response formats.

---

## Prompt 1: trading_analysis

**Requirement**: FR-002

**Description**: Analyze market conditions for a specific cryptocurrency and provide trading recommendations based on real-time market data.

**MCP Registration**:

```json
{
  "name": "trading_analysis",
  "description": "Analyze market conditions for a specific cryptocurrency and provide trading recommendations",
  "arguments": [
    {
      "name": "symbol",
      "description": "Trading pair symbol (e.g., BTCUSDT, ETHUSDT)",
      "required": true,
      "schema": {
        "type": "string",
        "pattern": "^[A-Z]{2,10}USDT$",
        "examples": ["BTCUSDT", "ETHUSDT", "BNBUSDT"]
      }
    },
    {
      "name": "strategy",
      "description": "Trading strategy preference: aggressive, balanced, or conservative",
      "required": false,
      "schema": {
        "type": "string",
        "enum": ["aggressive", "balanced", "conservative"]
      }
    },
    {
      "name": "risk_tolerance",
      "description": "Risk tolerance level: low, medium, or high",
      "required": false,
      "schema": {
        "type": "string",
        "enum": ["low", "medium", "high"]
      }
    }
  ]
}
```

**Request Format**:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "prompts/get",
  "params": {
    "name": "trading_analysis",
    "arguments": {
      "symbol": "BTCUSDT",
      "strategy": "balanced",
      "risk_tolerance": "medium"
    }
  }
}
```

**Success Response** (FR-005):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "# Market Analysis: BTCUSDT\n\n**Current Price**: $50,234.56\n**24h Change**: +2.5% ($1,234.56)\n**24h High**: $51,000.00\n**24h Low**: $49,000.00\n**24h Volume**: 12,345.67 BTC\n\n**Strategy Preference**: Balanced\n**Risk Tolerance**: Medium\n\n*Based on current market conditions, Bitcoin is showing positive momentum with a 2.5% gain over 24 hours. The price is holding above the daily low, suggesting bullish sentiment. For a balanced strategy with medium risk tolerance, consider:*\n\n*- Entry Point: Around $49,500-$50,000 (near current support)*\n*- Stop Loss: $48,500 (below 24h low)*\n*- Take Profit: $52,000 (near resistance)*\n\n*Last updated: 2025-10-17 14:23:45 UTC*"
        }
      }
    ]
  }
}
```

**Error Response - Invalid Symbol** (FR-007, FR-020):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32003,
    "message": "Invalid trading symbol 'INVALID'. Expected format: BTCUSDT, ETHUSDT",
    "data": {
      "provided_symbol": "INVALID",
      "valid_examples": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
      "recovery_suggestion": "Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT)"
    }
  }
}
```

**Error Response - Rate Limited** (FR-007, FR-018):

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Rate limit exceeded. Please wait 60 seconds before retrying.",
    "data": {
      "retry_after_secs": 60,
      "current_weight": 1200,
      "weight_limit": 1200,
      "recovery_suggestion": "Reduce request frequency or wait for rate limit window to reset"
    }
  }
}
```

**Acceptance Criteria** (User Story 1):

1. **Given** a user asks Claude "Should I buy Bitcoin now?" via prompt, **When** Claude invokes the trading_analysis prompt, **Then** the prompt returns formatted market data (price, 24h change, volume, high/low) with context for AI analysis

2. **Given** a user specifies aggressive strategy preference, **When** Claude invokes trading_analysis prompt with strategy parameter, **Then** the prompt includes strategy preference in the analysis context

3. **Given** market data is successfully retrieved, **When** Claude processes the prompt response, **Then** Claude provides actionable trading recommendation based on current conditions

---

## Prompt 2: portfolio_risk

**Requirement**: FR-004

**Description**: Retrieve account balances and format them for AI risk assessment and diversification recommendations.

**MCP Registration**:

```json
{
  "name": "portfolio_risk",
  "description": "Assess portfolio risk and provide diversification recommendations based on current holdings",
  "arguments": []
}
```

**Request Format**:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "prompts/get",
  "params": {
    "name": "portfolio_risk",
    "arguments": {}
  }
}
```

**Success Response** (FR-005):

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "# Portfolio Risk Assessment\n\n## Current Holdings\n\n| Asset | Free Balance | Locked Balance | Total | Est. USD Value |\n|-------|--------------|----------------|-------|----------------|\n| BTC   | 0.5000       | 0.0000         | 0.5000 | $25,117.28 |\n| ETH   | 5.2000       | 0.5000         | 5.7000 | $17,100.00 |\n| USDT  | 10,000.00    | 500.00         | 10,500.00 | $10,500.00 |\n| BNB   | 20.0000      | 0.0000         | 20.0000 | $6,000.00 |\n\n**Total Portfolio Value**: ~$58,717.28\n\n**Portfolio Composition**:\n- Bitcoin (BTC): 42.8% (High volatility)\n- Ethereum (ETH): 29.1% (Medium-high volatility)\n- Tether (USDT): 17.9% (Stable)\n- Binance Coin (BNB): 10.2% (Medium volatility)\n\n**Risk Assessment**: Your portfolio is heavily weighted toward high-volatility assets (71.9% in BTC+ETH). Consider the following for balanced diversification:\n\n*- Increase stablecoin allocation to 25-30% to reduce volatility*\n*- Diversify into additional altcoins to spread risk*\n*- Consider setting stop-loss orders on high-value positions*\n\n*Last updated: 2025-10-17 14:23:45 UTC*"
        }
      }
    ]
  }
}
```

**Error Response - Invalid Credentials** (FR-007, FR-019):

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32002,
    "message": "Invalid API credentials. Please check your BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables.",
    "data": {
      "masked_api_key": "AbCd****WxYz",
      "help_url": "https://testnet.binance.vision/",
      "recovery_suggestion": "Verify credentials at https://testnet.binance.vision/ and ensure correct environment variables"
    }
  }
}
```

**Edge Case - No Holdings**:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "# Portfolio Risk Assessment\n\n## Current Holdings\n\nNo active balances found in your account.\n\n**Recommendation**: Deposit funds to begin trading. Start with a diversified allocation:\n- 30% stablecoins (USDT/BUSD) for liquidity\n- 40% major cryptocurrencies (BTC/ETH)\n- 30% diversified altcoins based on risk tolerance\n\n*Last updated: 2025-10-17 14:23:45 UTC*"
        }
      }
    ]
  }
}
```

**Acceptance Criteria** (User Story 2):

1. **Given** a user has active balances in their account, **When** Claude invokes the portfolio_risk prompt, **Then** the prompt returns all non-zero balances formatted for AI analysis

2. **Given** account information is retrieved, **When** the prompt is processed, **Then** Claude receives both free and locked balances for each asset

3. **Given** portfolio data is presented, **When** Claude analyzes it, **Then** Claude provides risk assessment and diversification recommendations

---

## Capability Registration

**Requirement**: FR-006

**Server Capabilities Response**:

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "prompts": {
      "listChanged": false
    },
    "tools": {
      "listChanged": false
    }
  },
  "serverInfo": {
    "name": "mcp-binance-server",
    "version": "0.1.0"
  }
}
```

**Prompts List Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "prompts/list",
  "result": {
    "prompts": [
      {
        "name": "trading_analysis",
        "description": "Analyze market conditions for a specific cryptocurrency and provide trading recommendations",
        "arguments": [
          {
            "name": "symbol",
            "description": "Trading pair symbol (e.g., BTCUSDT, ETHUSDT)",
            "required": true
          },
          {
            "name": "strategy",
            "description": "Trading strategy preference: aggressive, balanced, or conservative",
            "required": false
          },
          {
            "name": "risk_tolerance",
            "description": "Risk tolerance level: low, medium, or high",
            "required": false
          }
        ]
      },
      {
        "name": "portfolio_risk",
        "description": "Assess portfolio risk and provide diversification recommendations based on current holdings",
        "arguments": []
      }
    ]
  }
}
```

---

## Implementation Notes

### Rust Handler Signature (FR-001):

```rust
use rmcp::{
    handler::server::ServerHandler,
    model::{GetPromptResult, PromptMessage, Role, TextContent, RequestContext, RoleServer},
    prompt_handler, Parameters,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[prompt_handler]
impl BinanceServer {
    #[prompt(
        name = "trading_analysis",
        description = "Analyze market conditions for a specific cryptocurrency and provide trading recommendations"
    )]
    async fn trading_analysis(
        &self,
        Parameters(args): Parameters<TradingAnalysisArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        // Implementation
    }

    #[prompt(
        name = "portfolio_risk",
        description = "Assess portfolio risk and provide diversification recommendations based on current holdings"
    )]
    async fn portfolio_risk(
        &self,
        Parameters(_args): Parameters<PortfolioRiskArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        // Implementation
    }
}
```

### Markdown Formatting Guidelines (FR-005, FR-007):

- Use H1 headers for main title (`# Market Analysis: BTCUSDT`)
- Use bold (**) for key metrics (price, change, volume)
- Use italics (*) for recommendations and timestamps
- Use tables for portfolio data (markdown table format)
- Include timestamp at end of all responses
- Escape special characters in dynamic content
- Format numbers with appropriate decimal places (2 for USD, 8 for crypto)

### Error Code Standards (FR-022):

| Error Type | Code | Description |
|------------|------|-------------|
| Rate Limited | -32001 | Binance API rate limit exceeded |
| Invalid Credentials | -32002 | API key/secret validation failed |
| Invalid Symbol | -32003 | Trading pair symbol format invalid |
| Insufficient Balance | -32004 | Account balance too low for operation |
| Internal Error | -32603 | Generic MCP internal error |

---

## Testing Strategy

### Unit Tests:

```rust
#[tokio::test]
async fn test_trading_analysis_valid_symbol() {
    let server = BinanceServer::new(/* ... */);
    let args = TradingAnalysisArgs {
        symbol: "BTCUSDT".to_string(),
        strategy: Some(TradingStrategy::Balanced),
        risk_tolerance: Some(RiskTolerance::Medium),
    };

    let result = server.trading_analysis(Parameters(args), ctx).await;
    assert!(result.is_ok());

    let prompt_result = result.unwrap();
    assert_eq!(prompt_result.messages.len(), 1);
    assert!(prompt_result.messages[0].content.as_text().contains("BTCUSDT"));
}

#[tokio::test]
async fn test_portfolio_risk_no_balances() {
    let server = BinanceServer::new(/* testnet with empty account */);
    let args = PortfolioRiskArgs {};

    let result = server.portfolio_risk(Parameters(args), ctx).await;
    assert!(result.is_ok());

    let content = result.unwrap().messages[0].content.as_text();
    assert!(content.contains("No active balances"));
}
```

### Integration Tests (Testnet):

1. Invoke trading_analysis with valid symbol → verify response format
2. Invoke trading_analysis with invalid symbol → verify error code -32003
3. Invoke portfolio_risk with valid credentials → verify balance table format
4. Invoke portfolio_risk with invalid credentials → verify error code -32002
5. Trigger rate limit → verify error code -32001 with retry_after

---

## Success Criteria Mapping

| Success Criterion | Contract Coverage |
|-------------------|-------------------|
| SC-001: Natural language questions answered within 3 seconds | trading_analysis prompt with < 3s response time (testnet latency) |
| SC-002: Portfolio risk assessment with complete breakdown | portfolio_risk prompt returns all non-zero balances with risk analysis |
| SC-007: Prompt responses include all market context | trading_analysis response contains price, volume, change, high/low formatted for LLM |

---

**Contract Status**: ✅ Complete - Ready for Implementation
