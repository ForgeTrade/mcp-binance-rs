# Error Handling Contract

**Feature**: [../spec.md](../spec.md)
**Requirements**: FR-017 to FR-024
**Created**: 2025-10-17

## Contract Overview

This document defines the enhanced error handling contract for the Binance MCP Server, including custom error types, recovery suggestions, and MCP error code mappings.

---

## Error Type 1: Rate Limited

**Requirement**: FR-017, FR-018

**Description**: Binance API rate limit exceeded. User must wait before retrying.

**Rust Definition**:

```rust
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimited {
        retry_after: Duration,     // Time to wait before retry
        current_weight: u32,       // Current rate limit weight consumed
        weight_limit: u32,         // Total weight limit per window
    },
    // ... other variants
}
```

**MCP Error Code**: `-32001`

**Error Response**:

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

**Trigger Conditions**:
- Binance API returns HTTP 429 (Too Many Requests)
- Binance API returns HTTP 418 (IP banned temporarily)
- `X-MBX-USED-WEIGHT` header exceeds `X-MBX-ORDER-COUNT` limit

**Recovery Suggestions** (FR-024):
1. "Wait {retry_after_secs} seconds before retrying"
2. "Reduce request frequency to stay under {weight_limit} weight per minute"
3. "Use resources instead of tools for frequently accessed data"
4. "Consider implementing exponential backoff for retries"

**Security Compliance** (FR-017):
- ✅ No API keys exposed in error message
- ✅ No internal stack traces
- ✅ Weight metrics safe to expose (public rate limit info)

**Acceptance Criteria** (User Story 5):

**Given** a rate limit is exceeded, **When** an API call fails with 429 status, **Then** the error message includes wait time and suggests reducing request frequency

---

## Error Type 2: Invalid Credentials

**Requirement**: FR-017, FR-019

**Description**: API key or secret key validation failed. User must verify credentials.

**Rust Definition**:

```rust
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("Invalid API credentials. Check environment variables")]
    InvalidCredentials {
        masked_key: String,        // First 4 + last 4 characters only
        help_url: String,          // Link to testnet documentation
    },
    // ... other variants
}
```

**MCP Error Code**: `-32002`

**Error Response**:

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

**Trigger Conditions**:
- Binance API returns HTTP 401 (Unauthorized)
- HMAC signature validation fails (HTTP 400 with error code -1022)
- Invalid API key format (missing or malformed)

**API Key Masking Logic** (FR-019, Security):

```rust
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    format!("{}****{}", &key[..4], &key[key.len()-4..])
}

// Example:
// Input:  "AbCdEfGhIjKlMnOpQrStUvWxYz"
// Output: "AbCd****WxYz"
```

**Recovery Suggestions** (FR-024):
1. "Check BINANCE_API_KEY environment variable is set correctly"
2. "Verify BINANCE_SECRET_KEY matches your API key"
3. "Ensure using testnet credentials for BINANCE_BASE_URL=https://testnet.binance.vision"
4. "Generate new API keys at {help_url} if credentials are compromised"

**Security Compliance** (FR-017, FR-023):
- ✅ Only first 4 + last 4 characters exposed (masked: `AbCd****WxYz`)
- ✅ Full API key NEVER appears in error message
- ✅ Secret key NEVER appears in any log or error
- ✅ User-friendly language ("check environment variables", not "HMAC signature mismatch")

**Acceptance Criteria** (User Story 5):

**Given** invalid API credentials are configured, **When** a tool is called, **Then** the error message indicates credential issue and suggests checking environment variables

---

## Error Type 3: Invalid Symbol

**Requirement**: FR-017, FR-020

**Description**: Trading pair symbol format is invalid or symbol does not exist on Binance.

**Rust Definition**:

```rust
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("Invalid trading symbol: {provided}")]
    InvalidSymbol {
        provided: String,          // Invalid symbol user provided
        format_help: String,       // Expected format description
        examples: Vec<String>,     // Valid symbol examples
    },
    // ... other variants
}
```

**MCP Error Code**: `-32003`

**Error Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32003,
    "message": "Invalid trading symbol 'BTC-USD'. Expected format: BTCUSDT, ETHUSDT",
    "data": {
      "provided_symbol": "BTC-USD",
      "format_help": "Use uppercase base asset + quote asset without separators",
      "valid_examples": ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "DOGEUSDT"],
      "recovery_suggestion": "Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT or btc/usdt)"
    }
  }
}
```

**Trigger Conditions**:
- Binance API returns HTTP 400 with error code -1121 ("Invalid symbol")
- Symbol contains invalid characters (hyphens, slashes, lowercase)
- Symbol not found in exchange info (non-existent trading pair)

**Common Invalid Formats**:
- `BTC-USDT` (hyphen separator)
- `btcusdt` (lowercase)
- `BTC/USDT` (slash separator)
- `BTCUSD` (wrong quote asset)
- `BTC` (missing quote asset)

**Recovery Suggestions** (FR-024):
1. "Use uppercase format: BTCUSDT, ETHUSDT, BNBUSDT"
2. "Remove separators (hyphens, slashes, spaces)"
3. "Ensure quote asset is USDT for spot trading"
4. "Check symbol exists on Binance testnet"

**Security Compliance**:
- ✅ No sensitive data exposed (symbol is user input)
- ✅ Helpful examples provided without overwhelming user

**Acceptance Criteria** (User Story 5):

**Given** an invalid symbol is provided (e.g., "INVALID"), **When** a market data tool is called, **Then** the error message suggests valid format and points to symbol documentation

---

## Error Type 4: Insufficient Balance

**Requirement**: FR-017, FR-021

**Description**: Account balance is too low to execute requested order.

**Rust Definition**:

```rust
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("Insufficient {asset} balance")]
    InsufficientBalance {
        asset: String,             // Asset name (BTC, USDT, etc.)
        required: String,          // Required amount (decimal string)
        available: String,         // Available amount (decimal string)
    },
    // ... other variants
}
```

**MCP Error Code**: `-32004`

**Error Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": -32004,
    "message": "Insufficient USDT balance. Required: 10000.00, Available: 5000.00",
    "data": {
      "asset": "USDT",
      "required_amount": "10000.00",
      "available_amount": "5000.00",
      "shortfall": "5000.00",
      "recovery_suggestion": "Deposit more funds or reduce order quantity to 5000.00 USDT or less"
    }
  }
}
```

**Trigger Conditions**:
- Binance API returns HTTP 400 with error code -2010 ("Insufficient balance")
- Attempting to place order exceeding available balance
- Locked balance prevents order execution

**Balance Formatting** (FR-021):

```rust
pub fn format_balance_error(
    asset: &str,
    required: &str,
    available: &str
) -> BinanceError {
    let req_decimal: f64 = required.parse().unwrap_or(0.0);
    let avail_decimal: f64 = available.parse().unwrap_or(0.0);
    let shortfall = req_decimal - avail_decimal;

    BinanceError::InsufficientBalance {
        asset: asset.to_string(),
        required: format!("{:.8}", req_decimal),
        available: format!("{:.8}", avail_decimal),
        shortfall: format!("{:.8}", shortfall),
    }
}
```

**Recovery Suggestions** (FR-024):
1. "Deposit at least {shortfall} {asset} to complete this order"
2. "Reduce order quantity to {available} {asset} or less"
3. "Cancel existing open orders to free up locked balance"
4. "Use market order instead of limit order to execute at current price"

**Security Compliance**:
- ✅ Balance amounts are user's own data (safe to expose)
- ✅ No other account information leaked

**Acceptance Criteria** (User Story 5):

**Given** insufficient balance for an order, **When** place_order is called, **Then** the error message shows required vs available balance with clear explanation

---

## Error Type 5: Generic API Error (Wrapper)

**Requirement**: FR-017, FR-022

**Description**: Wrapper for existing `reqwest::Error` and other API errors not covered by specific types.

**Rust Definition**:

```rust
#[derive(Debug, Error)]
pub enum BinanceError {
    /// Wrapper for existing API errors (backward compatibility)
    #[error("Binance API error: {0}")]
    ApiError(#[from] reqwest::Error),
}
```

**MCP Error Code**: `-32603` (Internal Error)

**Error Response**:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32603,
    "message": "Binance API error: Connection timeout",
    "data": {
      "error_type": "reqwest::Error",
      "recovery_suggestion": "Check network connectivity and retry. If issue persists, Binance API may be experiencing downtime."
    }
  }
}
```

**Trigger Conditions**:
- Network connectivity issues (DNS, TCP connection)
- SSL/TLS certificate errors
- HTTP timeout (exceeds reqwest timeout)
- Binance API returns 5xx server errors
- Any error not matching specific types above

**Security Compliance** (FR-017, FR-023):
- ✅ No internal paths exposed
- ✅ No stack traces in production logs
- ✅ Error details sanitized before conversion to ErrorData

---

## Error Code Standards

**Requirement**: FR-022

| Error Code | Error Type | Description |
|------------|------------|-------------|
| -32001 | RateLimited | Binance API rate limit exceeded |
| -32002 | InvalidCredentials | API key/secret validation failed |
| -32003 | InvalidSymbol | Trading pair symbol format invalid |
| -32004 | InsufficientBalance | Account balance too low for operation |
| -32404 | ResourceNotFound | Resource URI invalid or not found |
| -32603 | InternalError | Generic API error (reqwest, network, etc.) |

**MCP Standard Codes** (Used for reference):
- `-32700`: Parse error (Invalid JSON)
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

**Custom Code Range**: `-32001` to `-32099` (Binance-specific errors)

---

## Error Conversion Implementation

**Requirement**: FR-022

**From<BinanceError> for ErrorData**:

```rust
use rmcp::model::{ErrorData, ErrorCode};
use serde_json::json;

impl From<BinanceError> for ErrorData {
    fn from(err: BinanceError) -> Self {
        match err {
            BinanceError::RateLimited { retry_after, current_weight, weight_limit } => {
                ErrorData::new(
                    ErrorCode::Custom(-32001),
                    format!("Rate limit exceeded. Please wait {} seconds before retrying.", retry_after.as_secs())
                )
                .with_data(json!({
                    "retry_after_secs": retry_after.as_secs(),
                    "current_weight": current_weight,
                    "weight_limit": weight_limit,
                    "recovery_suggestion": "Reduce request frequency or wait for rate limit window to reset"
                }))
            },

            BinanceError::InvalidCredentials { masked_key, help_url } => {
                ErrorData::new(
                    ErrorCode::Custom(-32002),
                    "Invalid API credentials. Please check your BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables."
                )
                .with_data(json!({
                    "masked_api_key": masked_key,
                    "help_url": help_url,
                    "recovery_suggestion": "Verify credentials at https://testnet.binance.vision/ and ensure correct environment variables"
                }))
            },

            BinanceError::InvalidSymbol { provided, format_help, examples } => {
                ErrorData::new(
                    ErrorCode::Custom(-32003),
                    format!("Invalid trading symbol '{}'. {}", provided, format_help)
                )
                .with_data(json!({
                    "provided_symbol": provided,
                    "format_help": format_help,
                    "valid_examples": examples,
                    "recovery_suggestion": "Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT)"
                }))
            },

            BinanceError::InsufficientBalance { asset, required, available } => {
                ErrorData::new(
                    ErrorCode::Custom(-32004),
                    format!("Insufficient {} balance. Required: {}, Available: {}", asset, required, available)
                )
                .with_data(json!({
                    "asset": asset,
                    "required_amount": required,
                    "available_amount": available,
                    "recovery_suggestion": "Deposit more funds or reduce order quantity"
                }))
            },

            BinanceError::ApiError(e) => {
                ErrorData::new(
                    ErrorCode::InternalError,
                    format!("Binance API error: {}", e)
                )
                .with_data(json!({
                    "error_type": "reqwest::Error",
                    "recovery_suggestion": "Check network connectivity and retry"
                }))
            },
        }
    }
}
```

---

## User-Friendly Error Messages

**Requirement**: FR-023, FR-024

### Before Enhancement (Current):

```
Error: reqwest error: Connection refused
Error: HTTP 429: Too Many Requests
Error: Invalid symbol
```

### After Enhancement (Phase 1):

```
Rate limit exceeded. Please wait 60 seconds before retrying.
→ Recovery: Reduce request frequency or wait for rate limit window to reset

Invalid API credentials. Please check your BINANCE_API_KEY and BINANCE_SECRET_KEY environment variables.
→ Recovery: Verify credentials at https://testnet.binance.vision/

Invalid trading symbol 'BTC-USD'. Expected format: BTCUSDT, ETHUSDT
→ Recovery: Use uppercase symbols without separators (e.g., BTCUSDT, not BTC-USDT)

Insufficient USDT balance. Required: 10000.00, Available: 5000.00
→ Recovery: Deposit more funds or reduce order quantity to 5000.00 USDT or less
```

**Language Guidelines** (FR-023):
- ✅ Use "you/your" instead of "user"
- ✅ Explain "what" and "why", not just "error"
- ✅ Provide specific recovery steps
- ✅ Avoid technical jargon (HMAC, SHA256, etc.)
- ✅ Include relevant URLs for documentation
- ❌ No stack traces or internal paths
- ❌ No raw Binance error codes without explanation

---

## Testing Strategy

### Unit Tests:

```rust
#[test]
fn test_mask_api_key() {
    assert_eq!(mask_api_key("AbCdEfGhIjKlMnOpQrStUvWxYz"), "AbCd****WxYz");
    assert_eq!(mask_api_key("short"), "*****");
    assert_eq!(mask_api_key(""), "");
}

#[test]
fn test_rate_limited_error_conversion() {
    let err = BinanceError::RateLimited {
        retry_after: Duration::from_secs(60),
        current_weight: 1200,
        weight_limit: 1200,
    };

    let error_data = ErrorData::from(err);
    assert_eq!(error_data.code, ErrorCode::Custom(-32001));
    assert!(error_data.message.contains("60 seconds"));
}

#[test]
fn test_insufficient_balance_error() {
    let err = BinanceError::InsufficientBalance {
        asset: "USDT".to_string(),
        required: "10000.00".to_string(),
        available: "5000.00".to_string(),
    };

    let error_data = ErrorData::from(err);
    assert_eq!(error_data.code, ErrorCode::Custom(-32004));
    assert!(error_data.message.contains("Required: 10000.00"));
    assert!(error_data.data.is_some());
}
```

### Integration Tests (Testnet):

1. **Trigger rate limit** → Call get_ticker 100 times rapidly → Verify error code -32001 with retry_after
2. **Use invalid credentials** → Set wrong API key → Verify error code -32002 with masked key
3. **Use invalid symbol** → Call get_ticker("BTC-USD") → Verify error code -32003 with examples
4. **Trigger insufficient balance** → Place large order → Verify error code -32004 with amounts
5. **Network timeout** → Disconnect network → Verify error code -32603 with recovery suggestion

---

## Success Criteria Mapping

| Success Criterion | Contract Coverage |
|-------------------|-------------------|
| SC-004: 90% of error scenarios provide recovery suggestions | 4 specific errors + 1 generic = 5 types, all with suggestions |
| SC-006: 60% reduction in user confusion | User-friendly language, specific steps, no jargon |

---

## Security & Compliance

### Security Checklist (FR-017, Constitution § Security-First):

- [x] No full API keys in error messages (masked: `AbCd****WxYz`)
- [x] No secret keys in any log or error
- [x] No internal paths or stack traces exposed
- [x] No sensitive account data (only user's own balance/orders)
- [x] Rate limit metrics safe to expose (public info)
- [x] Error messages don't guide malicious users (no SQL injection hints, etc.)

### User-Friendly Language (FR-023):

- [x] Avoid: "HMAC signature invalid", "Signature mismatch"
  → Use: "Invalid API credentials. Check environment variables"

- [x] Avoid: "Error -1121: Invalid symbol"
  → Use: "Invalid trading symbol 'BTC-USD'. Expected format: BTCUSDT"

- [x] Avoid: "HTTP 429"
  → Use: "Rate limit exceeded. Please wait 60 seconds"

- [x] Avoid: "Insufficient funds"
  → Use: "Insufficient USDT balance. Required: 10000, Available: 5000"

### Actionable Recovery (FR-024):

Every error MUST include:
1. **What happened**: Clear description of the error
2. **Why it happened**: Context (rate limit, wrong format, etc.)
3. **How to fix**: Specific, actionable steps
4. **Where to learn more**: URLs to documentation (optional)

---

**Contract Status**: ✅ Complete - Ready for Implementation
