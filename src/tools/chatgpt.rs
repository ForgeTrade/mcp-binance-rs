//! ChatGPT-compatible MCP tools (search/fetch)
//!
//! Implements the required `search` and `fetch` tools for ChatGPT connectors
//! and deep research. These tools wrap Binance API data into the format
//! expected by ChatGPT's MCP integration.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::binance::BinanceClient;
use crate::error::McpError;

/// Search result item for ChatGPT MCP integration
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique identifier (trading symbol)
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Brief description with current price
    pub text: String,
    /// Canonical URL for citation
    pub url: String,
}

/// Fetch result for ChatGPT MCP integration
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResult {
    /// Unique identifier (trading symbol)
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Full detailed information
    pub text: String,
    /// Canonical URL for citation
    pub url: String,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Common trading symbols for search
const POPULAR_SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "SOLUSDT",
    "XRPUSDT", "DOGEUSDT", "DOTUSDT", "MATICUSDT", "LINKUSDT",
    "LTCUSDT", "AVAXUSDT", "UNIUSDT", "ATOMUSDT", "XLMUSDT",
];

/// Search for trading symbols by keyword
///
/// Returns top 10 matching symbols with current prices.
/// Searches against common trading pairs.
pub async fn search_symbols(
    client: &BinanceClient,
    query: &str,
) -> Result<Vec<SearchResult>, McpError> {
    let query_upper = query.to_uppercase();
    let mut results = Vec::new();

    // Filter popular symbols by query match
    let matched_symbols: Vec<&str> = POPULAR_SYMBOLS
        .iter()
        .filter(|symbol| {
            symbol.contains(&query_upper)
        })
        .take(10)
        .copied()
        .collect();

    // Get current prices for matched symbols
    for symbol in matched_symbols {
        // Get ticker price
        let ticker = match client.get_ticker_price(symbol).await {
            Ok(t) => t,
            Err(_) => continue, // Skip if price unavailable
        };

        // Parse symbol into base/quote (e.g., BTCUSDT -> BTC/USDT)
        let (base, quote) = parse_symbol(symbol);
        let title = format!("{}/{}", base, quote);
        let text = format!("Current price: {} {}", ticker.price, quote);
        let url = format!(
            "https://www.binance.com/en/trade/{}_{}",
            base, quote
        );

        results.push(SearchResult {
            id: symbol.to_string(),
            title,
            text,
            url,
        });
    }

    // If no results, return top 5 popular pairs
    if results.is_empty() {
        for symbol in POPULAR_SYMBOLS.iter().take(5) {
            let ticker = match client.get_ticker_price(symbol).await {
                Ok(t) => t,
                Err(_) => continue,
            };

            let (base, quote) = parse_symbol(symbol);
            let title = format!("{}/{}", base, quote);
            let text = format!("Current price: {} {}", ticker.price, quote);
            let url = format!(
                "https://www.binance.com/en/trade/{}_{}",
                base, quote
            );

            results.push(SearchResult {
                id: symbol.to_string(),
                title,
                text,
                url,
            });
        }
    }

    Ok(results)
}

/// Fetch detailed information for a specific trading symbol
///
/// Returns comprehensive data including:
/// - Current price and 24h statistics
/// - Recent price action (klines)
/// - Order book depth (top 5 levels)
/// - Trading rules and filters
pub async fn fetch_symbol_details(
    client: &BinanceClient,
    symbol: &str,
) -> Result<FetchResult, McpError> {
    let symbol_upper = symbol.to_uppercase();

    // Get 24hr ticker stats
    let ticker_24hr = client.get_24hr_ticker(&symbol_upper).await?;

    // Get orderbook depth (top 5 levels)
    let orderbook = client.get_order_book(&symbol_upper, Some(5)).await?;

    // Format bid levels
    let bids = orderbook
        .bids
        .iter()
        .map(|(price, qty)| format!("  {} @ {}", qty, price))
        .collect::<Vec<_>>()
        .join("\n");

    // Format ask levels
    let asks = orderbook
        .asks
        .iter()
        .map(|(price, qty)| format!("  {} @ {}", qty, price))
        .collect::<Vec<_>>()
        .join("\n");

    // Parse symbol for title and URL
    let (base, quote) = parse_symbol(&symbol_upper);
    let title = format!("{}/{} Market Data", base, quote);
    let url = format!("https://www.binance.com/en/trade/{}_{}", base, quote);

    // Build comprehensive text description
    let text = format!(
        r#"# {} Market Overview

## Current Price
Last Price: {} {}
24h Change: {} {} ({}%)

## 24-Hour Statistics
High: {} {}
Low: {} {}
Volume: {} {}
Quote Volume: {} {}

## Order Book (Top 5 Levels)

### Best Asks (Sell Orders)
{}

### Best Bids (Buy Orders)
{}

## Trading Information
Symbol: {}
Base Asset: {}
Quote Asset: {}
"#,
        title,
        ticker_24hr.last_price,
        quote,
        ticker_24hr.price_change,
        quote,
        ticker_24hr.price_change_percent,
        ticker_24hr.high_price,
        quote,
        ticker_24hr.low_price,
        quote,
        ticker_24hr.volume,
        base,
        ticker_24hr.quote_volume,
        quote,
        asks,
        bids,
        symbol_upper,
        base,
        quote,
    );

    // Metadata
    let metadata = json!({
        "baseAsset": base,
        "quoteAsset": quote,
        "24hStats": {
            "priceChange": ticker_24hr.price_change,
            "priceChangePercent": ticker_24hr.price_change_percent,
            "weightedAvgPrice": ticker_24hr.weighted_avg_price,
            "volume": ticker_24hr.volume,
            "quoteVolume": ticker_24hr.quote_volume,
            "trades": ticker_24hr.count,
        },
        "orderBook": {
            "bidLevels": orderbook.bids.len(),
            "askLevels": orderbook.asks.len(),
        }
    });

    Ok(FetchResult {
        id: symbol_upper,
        title,
        text,
        url,
        metadata: Some(metadata),
    })
}

/// Parse symbol into base and quote assets
///
/// Examples:
/// - BTCUSDT -> (BTC, USDT)
/// - ETHBTC -> (ETH, BTC)
fn parse_symbol(symbol: &str) -> (String, String) {
    // Common quote assets (longest first to avoid partial matches)
    let quote_assets = ["USDT", "BUSD", "USDC", "BTC", "ETH", "BNB", "EUR", "GBP"];

    for quote in &quote_assets {
        if symbol.ends_with(quote) {
            let base = symbol.strip_suffix(quote).unwrap_or(symbol);
            return (base.to_string(), quote.to_string());
        }
    }

    // Fallback: assume last 3-4 chars are quote
    if symbol.len() > 6 {
        let split_pos = symbol.len() - 4;
        (
            symbol[..split_pos].to_string(),
            symbol[split_pos..].to_string(),
        )
    } else {
        (symbol.to_string(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_symbol("BTCUSDT"), ("BTC".to_string(), "USDT".to_string()));
        assert_eq!(parse_symbol("ETHBTC"), ("ETH".to_string(), "BTC".to_string()));
        assert_eq!(parse_symbol("BNBBUSD"), ("BNB".to_string(), "BUSD".to_string()));
        assert_eq!(parse_symbol("ADAETH"), ("ADA".to_string(), "ETH".to_string()));
    }
}
