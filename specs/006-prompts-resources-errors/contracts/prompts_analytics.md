# Advanced Analytics Prompts Contract

**Feature**: [../spec.md](../spec.md) + Feature 008 (Advanced Order Book Analytics)
**Requirements**: FR-002 + Feature 008 analytics capabilities
**Created**: 2025-01-18

## Contract Overview

This document defines MCP prompts that leverage the advanced order book analytics from Feature 008, providing comprehensive market microstructure analysis for professional trading decisions.

---

## Prompt 1: advanced_market_analysis

**Description**: Comprehensive market analysis combining orderbook analytics, order flow, volume profile, anomaly detection, and health scoring.

**MCP Registration**:

```json
{
  "name": "advanced_market_analysis",
  "description": "Perform deep market microstructure analysis using order flow, volume profile, anomaly detection, and health scoring",
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
      "name": "analysis_depth",
      "description": "Analysis depth: quick (5min), standard (1h), deep (24h)",
      "required": false,
      "schema": {
        "type": "string",
        "enum": ["quick", "standard", "deep"],
        "default": "standard"
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
    "name": "advanced_market_analysis",
    "arguments": {
      "symbol": "BTCUSDT",
      "analysis_depth": "standard"
    }
  }
}
```

**Success Response**:

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
          "text": "# Advanced Market Analysis: BTCUSDT\n\n**Analysis Time**: 2025-01-18 14:23:45 UTC\n**Analysis Depth**: Standard (1 hour)\n\n---\n\n## 1. Order Flow Analysis (Last 60 seconds)\n\n**Flow Direction**: **StrongBuy** 📈\n- Bid Flow Rate: 12.5 orders/sec\n- Ask Flow Rate: 4.2 orders/sec\n- Net Flow: +8.3 orders/sec (bid pressure dominant)\n- Cumulative Delta: +145.7 BTC\n\n*Interpretation*: Strong buying pressure with bid flow 3x higher than ask flow. Institutional accumulation likely occurring.*\n\n---\n\n## 2. Volume Profile (Last 24 hours)\n\n**Key Price Levels:**\n- **POC (Point of Control)**: $50,234.50 - Maximum volume traded\n- **VAH (Value Area High)**: $51,100.00 - 70% volume upper bound\n- **VAL (Value Area Low)**: $49,500.00 - 70% volume lower bound\n\n**Support/Resistance Zones:**\n- **Strong Support**: $49,500 - $49,800 (VAL zone, 15% of 24h volume)\n- **Strong Resistance**: $51,000 - $51,200 (VAH zone, 12% of 24h volume)\n\n*Trading Strategy*: Price is currently above POC ($50,234), suggesting bullish sentiment. Entry near VAL for long positions offers best risk/reward.*\n\n---\n\n## 3. Market Microstructure Health\n\n**Overall Health Score**: **78/100** (Good) ✅\n\n**Component Breakdown:**\n- Spread Stability: 85/100 (Excellent - low volatility)\n- Liquidity Depth: 72/100 (Good - 72 avg levels)\n- Flow Balance: 68/100 (Fair - slight buy bias)\n- Update Rate: 88/100 (Excellent - 45 updates/sec)\n\n**Trading Recommendation**: *Good market conditions for active trading. Normal position sizes recommended. Market is liquid and stable with manageable volatility.*\n\n---\n\n## 4. Anomaly Detection\n\n**Detected Anomalies**: 1\n\n### Iceberg Order Detected (Medium Severity) ⚠️\n- **Price Level**: $50,100.00\n- **Absorbed Volume**: 18.5 BTC\n- **Refill Count**: 7 times\n- **Refill Rate**: 7.2x median (threshold: 5x)\n\n*Action*: Large hidden institutional order at $50,100. This level will act as strong support. Consider this a \"smart money\" accumulation zone.*\n\n**No Critical Anomalies** (Quote stuffing or flash crash risk not detected)\n\n---\n\n## 5. Liquidity Vacuum Analysis\n\n**Detected Vacuums**: 2\n\n### Vacuum 1: Fast Movement Risk 🔥\n- **Price Range**: $51,500 - $52,200\n- **Volume Deficit**: 87% below median\n- **Expected Impact**: FastMovement\n- *If price breaks above $51,500, expect rapid move to $52,200 due to low liquidity*\n\n### Vacuum 2: Moderate Movement Risk\n- **Price Range**: $48,800 - $49,200\n- **Volume Deficit**: 62% below median\n- **Expected Impact**: ModerateMovement\n- *Downside vacuum - price may gap down quickly if $48,800 breaks*\n\n---\n\n## 6. Actionable Trading Recommendations\n\n### For Long (Buy) Positions:\n- ✅ **Entry Zone**: $49,500 - $49,800 (VAL support + high volume)\n- ✅ **Stop Loss**: $48,700 (below downside vacuum)\n- ✅ **Take Profit**: $51,000 (VAH resistance, before liquidity vacuum)\n- ⚠️ **Risk**: If price breaks $51,500, may gap to $52,200 (liquidity vacuum)\n\n### For Short (Sell) Positions:\n- ⚠️ **Not Recommended** - Strong buy flow and iceberg support at $50,100\n- If shorting: Entry only above $51,200 with tight stops\n\n### Market Conditions Summary:\n- **Bias**: Bullish (StrongBuy flow direction)\n- **Risk Level**: Medium (Good health score 78/100)\n- **Best Strategy**: Long entries near VAL with targets at VAH\n- **Hidden Smart Money**: Iceberg order at $50,100 (institutional accumulation)\n\n---\n\n*Analysis generated using 5 advanced analytics tools: order_flow, volume_profile, anomaly_detection, liquidity_vacuums, health_scoring*\n\n*Last updated: 2025-01-18 14:23:45 UTC*"
        }
      }
    ]
  }
}
```

**Acceptance Criteria**:

1. **Given** user requests deep market analysis, **When** prompt is invoked, **Then** all 5 analytics tools are called and results are synthesized
2. **Given** anomalies are detected, **When** prompt formats response, **Then** severity levels and actionable recommendations are included
3. **Given** liquidity vacuums exist, **When** prompt is processed, **Then** price ranges with gap risk are highlighted
4. **Given** health score is calculated, **When** response is formatted, **Then** component breakdown and trading guidance are provided

---

## Prompt 2: orderflow_snapshot

**Description**: Quick order flow snapshot for real-time trading decisions.

**MCP Registration**:

```json
{
  "name": "orderflow_snapshot",
  "description": "Get instant order flow direction and bid/ask pressure for rapid trading decisions",
  "arguments": [
    {
      "name": "symbol",
      "description": "Trading pair symbol (e.g., BTCUSDT)",
      "required": true,
      "schema": {
        "type": "string",
        "pattern": "^[A-Z]{2,10}USDT$"
      }
    },
    {
      "name": "window_secs",
      "description": "Analysis window in seconds (default: 60)",
      "required": false,
      "schema": {
        "type": "integer",
        "minimum": 10,
        "maximum": 300,
        "default": 60
      }
    }
  ]
}
```

**Success Response**:

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
          "text": "# Order Flow Snapshot: BTCUSDT\n\n**Window**: Last 60 seconds\n**Timestamp**: 2025-01-18 14:23:45 UTC\n\n## Flow Direction: **StrongBuy** 📈\n\n**Flow Metrics:**\n- Bid Flow: 12.5 orders/sec\n- Ask Flow: 4.2 orders/sec\n- Net Flow: +8.3 (strong buy pressure)\n- Cumulative Delta: +145.7 BTC\n\n**Quick Take**: Aggressive buying. Bid flow 3x stronger than ask flow. Bullish momentum confirmed.\n\n**Action**: Consider long entries if aligned with strategy. Strong institutional demand evident."
        }
      }
    ]
  }
}
```

---

## Prompt 3: market_health_check

**Description**: Rapid market health assessment before entering trades.

**MCP Registration**:

```json
{
  "name": "market_health_check",
  "description": "Quick health check of market conditions before trading",
  "arguments": [
    {
      "name": "symbol",
      "description": "Trading pair symbol",
      "required": true,
      "schema": {
        "type": "string",
        "pattern": "^[A-Z]{2,10}USDT$"
      }
    }
  ]
}
```

**Success Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "# Market Health: BTCUSDT\n\n**Overall Score**: **78/100** ✅ **GOOD**\n\n**Status**: Safe to trade with normal position sizes\n\n**Breakdown:**\n- ✅ Spread Stability: 85/100 (Low volatility)\n- ✅ Liquidity: 72/100 (Adequate depth)\n- ⚠️ Flow Balance: 68/100 (Slight buy bias)\n- ✅ Activity: 88/100 (Healthy update rate)\n\n**Risk Assessment**: Low-medium risk. No critical issues detected. Standard risk management applies.\n\n**Anomalies**: None critical. 1 iceberg order detected (normal institutional activity)."
        }
      }
    ]
  }
}
```

---

## Implementation Notes

### Rust Handler Signature:

```rust
use rmcp::{
    handler::server::ServerHandler,
    model::{GetPromptResult, PromptMessage, Role, TextContent, RequestContext, RoleServer},
    prompt_handler, Parameters,
};

#[cfg(feature = "orderbook_analytics")]
use crate::orderbook::analytics::{
    flow::calculate_order_flow,
    profile::generate_volume_profile,
    anomaly::detect_anomalies,
    health::calculate_health_score,
    storage::SnapshotStorage,
};

#[prompt_handler]
impl BinanceServer {
    #[prompt(
        name = "advanced_market_analysis",
        description = "Perform deep market microstructure analysis"
    )]
    #[cfg(feature = "orderbook_analytics")]
    async fn advanced_market_analysis(
        &self,
        Parameters(args): Parameters<AdvancedAnalysisArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let symbol = &args.symbol;
        let storage = &self.snapshot_storage;

        // 1. Get order flow (60 seconds)
        let order_flow = calculate_order_flow(storage, symbol, 60, None).await?;

        // 2. Get volume profile (24 hours)
        let tick_size = Decimal::from_str("0.01")?;
        let volume_profile = generate_volume_profile(symbol, 24, tick_size).await?;

        // 3. Detect anomalies (60 seconds)
        let anomalies = detect_anomalies(storage, symbol, 60).await?;

        // 4. Get health score (300 seconds / 5 minutes)
        let health = calculate_health_score(storage, symbol, 300).await?;

        // 5. Identify liquidity vacuums (from volume profile)
        let vacuums = identify_liquidity_vacuums(&volume_profile)?;

        // Format comprehensive markdown response
        let markdown = format_advanced_analysis(
            symbol,
            &order_flow,
            &volume_profile,
            &anomalies,
            &health,
            &vacuums,
        );

        Ok(GetPromptResult {
            messages: vec![PromptMessage {
                role: Role::User,
                content: PromptContent::text(markdown),
            }],
        })
    }
}
```

### Response Formatting Guidelines:

1. **Structure**: Use H1 for main title, H2 for sections, H3 for subsections
2. **Emojis**: Use sparingly for visual cues (📈 bullish, 📉 bearish, ⚠️ warning, ✅ good, 🔥 critical)
3. **Formatting**:
   - Bold (`**`) for key metrics and recommendations
   - Italics (`*`) for interpretations and context
   - Tables for structured data
   - Bullet points for lists
4. **Sections Order**:
   1. Order Flow (immediate pressure)
   2. Volume Profile (support/resistance)
   3. Health Score (market conditions)
   4. Anomalies (risks/opportunities)
   5. Liquidity Vacuums (gap risk zones)
   6. Actionable Recommendations (synthesis)
5. **Numbers**:
   - Prices: 2 decimal places ($50,234.50)
   - Percentages: 1 decimal (78.5%)
   - Volume: 1-2 decimals (145.7 BTC)
   - Scores: Integer (78/100)

---

## Testing Strategy

### Unit Tests:

```rust
#[tokio::test]
#[cfg(feature = "orderbook_analytics")]
async fn test_advanced_market_analysis_integration() {
    let server = BinanceServer::new_with_storage(/* mock storage */);
    let args = AdvancedAnalysisArgs {
        symbol: "BTCUSDT".to_string(),
        analysis_depth: AnalysisDepth::Standard,
    };

    let result = server.advanced_market_analysis(Parameters(args), ctx).await;
    assert!(result.is_ok());

    let content = result.unwrap().messages[0].content.as_text();

    // Verify all sections present
    assert!(content.contains("Order Flow Analysis"));
    assert!(content.contains("Volume Profile"));
    assert!(content.contains("Market Microstructure Health"));
    assert!(content.contains("Anomaly Detection"));
    assert!(content.contains("Liquidity Vacuum Analysis"));
    assert!(content.contains("Actionable Trading Recommendations"));
}
```

---

## Success Criteria

| Criterion | Implementation |
|-----------|----------------|
| Comprehensive analysis | All 5 analytics tools integrated in single prompt |
| Actionable insights | Specific entry/exit/stop recommendations provided |
| Risk awareness | Anomalies and vacuums highlighted with severity |
| Performance | Total response time <5 seconds for standard depth |
| Professional format | Clear sections with visual hierarchy |

---

**Contract Status**: ✅ Ready for Implementation
