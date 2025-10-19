# Quickstart: Mainnet Support with Secure API Key Authentication

**Date**: 2025-10-19
**Branch**: `011-mainnet-api-auth`

## Overview

This quickstart demonstrates configuring Binance API credentials for a ChatGPT MCP session and using authenticated account/trading tools. Covers both testnet (testing) and mainnet (production) environments.

**Time to Complete**: 5 minutes
**Prerequisites**:
- Binance account with API key/secret (testnet or mainnet)
- MCP server deployed and accessible via ChatGPT

---

## Scenario 1: Configure Testnet Credentials (P1 - Core)

**Goal**: Configure testnet API credentials and verify account access

### Step 1: Get Your Testnet API Credentials

1. Go to https://testnet.binance.vision/
2. Log in with GitHub account
3. Generate API Key → Copy API Key and Secret Key

**Example credentials** (these are fake, use your own):
```
API Key:    1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
API Secret: fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321
```

### Step 2: Configure Credentials via ChatGPT

**Prompt**:
```
Configure my Binance testnet credentials:
API Key: [paste your 64-char key]
API Secret: [paste your 64-char secret]
Environment: testnet
```

**Expected MCP Tool Call**:
```json
{
  "tool": "configure_credentials",
  "arguments": {
    "api_key": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "api_secret": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
    "environment": "testnet"
  }
}
```

**Expected Response**:
```json
{
  "configured": true,
  "environment": "testnet",
  "configured_at": "2025-10-19T10:30:45Z"
}
```

**Success Criteria** (SC-004): Credentials configured within 100ms

### Step 3: Verify Configuration

**Prompt**:
```
What are my current credential settings?
```

**Expected MCP Tool Call**:
```json
{
  "tool": "get_credentials_status",
  "arguments": {}
}
```

**Expected Response**:
```json
{
  "configured": true,
  "environment": "testnet",
  "key_prefix": "12345678",
  "configured_at": "2025-10-19T10:30:45Z"
}
```

**Validation**:
✅ `configured: true`
✅ `environment: "testnet"`
✅ `key_prefix` shows first 8 chars of your API key (NFR-003 security requirement)

### Step 4: Test Account Access

**Prompt**:
```
Show my Binance account information
```

**Expected MCP Tool Call**:
```json
{
  "tool": "get_account_info",
  "arguments": {}
}
```

**Expected Response** (example):
```json
{
  "accountType": "SPOT",
  "canTrade": true,
  "canWithdraw": false,
  "canDeposit": false,
  "balances": [
    {"asset": "BTC", "free": "1000.00000000", "locked": "0.00000000"},
    {"asset": "USDT", "free": "10000.00000000", "locked": "0.00000000"}
  ],
  "permissions": ["SPOT"]
}
```

**Success Criteria** (SC-001): Account info returns valid data (not CREDENTIALS_NOT_CONFIGURED error)

**Validation**:
✅ Response shows testnet balances (typically 1000 BTC, 10000 USDT for new testnet accounts)
✅ `canTrade: true` indicates API key has trading permissions
✅ Request went to `https://testnet.binance.vision` (testnet endpoint per FR-002)

---

## Scenario 2: Switch to Mainnet (P2 - Environment Switching)

**Goal**: Switch from testnet to mainnet environment

### Step 1: Get Your Mainnet API Credentials

1. Go to https://www.binance.com/en/my/settings/api-management
2. Create API Key → Enable "Enable Spot & Margin Trading" permission
3. Copy API Key and Secret Key

**⚠️ WARNING**: Mainnet credentials control real funds. Use testnet for practice!

### Step 2: Reconfigure with Mainnet Credentials

**Prompt**:
```
Switch to mainnet with my production API credentials:
API Key: [paste your mainnet 64-char key]
API Secret: [paste your mainnet 64-char secret]
Environment: mainnet
```

**Expected MCP Tool Call**:
```json
{
  "tool": "configure_credentials",
  "arguments": {
    "api_key": "<your_mainnet_key>",
    "api_secret": "<your_mainnet_secret>",
    "environment": "mainnet"
  }
}
```

**Expected Response**:
```json
{
  "configured": true,
  "environment": "mainnet",
  "configured_at": "2025-10-19T10:35:22Z"
}
```

**Success Criteria** (SC-005): Environment switch takes effect within 1 API call

### Step 3: Verify Mainnet Environment

**Prompt**:
```
What environment am I using?
```

**Expected Response**:
```json
{
  "configured": true,
  "environment": "mainnet",
  "key_prefix": "abcdefgh",
  "configured_at": "2025-10-19T10:35:22Z"
}
```

**Validation**:
✅ `environment: "mainnet"` (changed from "testnet")
✅ `configured_at` timestamp is newer (last write wins per edge case handling)

### Step 4: Verify Real Account Balances

**Prompt**:
```
Show my real Binance account balances
```

**Expected MCP Tool Call**:
```json
{
  "tool": "get_account_info",
  "arguments": {}
}
```

**Expected Response** (example):
```json
{
  "accountType": "SPOT",
  "canTrade": true,
  "balances": [
    {"asset": "BTC", "free": "0.05000000", "locked": "0.00000000"},
    {"asset": "ETH", "free": "1.20000000", "locked": "0.00000000"},
    {"asset": "USDT", "free": "500.00000000", "locked": "0.00000000"}
  ]
}
```

**Validation**:
✅ Balances show real account data (different from testnet's fake balances)
✅ Request went to `https://api.binance.com` (mainnet endpoint per FR-002)
✅ Assets reflect your actual holdings

---

## Scenario 3: Handle Invalid Credentials (Edge Case Testing)

**Goal**: Verify structured error responses for invalid credentials

### Test 3a: Invalid API Key Format

**Prompt**:
```
Configure credentials with a short API key:
API Key: short_key
API Secret: fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321
Environment: testnet
```

**Expected MCP Tool Call**:
```json
{
  "tool": "configure_credentials",
  "arguments": {
    "api_key": "short_key",
    "api_secret": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
    "environment": "testnet"
  }
}
```

**Expected Error Response** (FR-009, FR-013):
```json
{
  "error_code": "INVALID_API_KEY_FORMAT",
  "message": "API key must be exactly 64 alphanumeric characters"
}
```

**Success Criteria** (SC-007): Error detected synchronously within 10ms

**Validation**:
✅ Error returned immediately (synchronous format validation)
✅ Error code is machine-readable: `INVALID_API_KEY_FORMAT`
✅ Human-readable message explains the requirement

### Test 3b: Invalid Environment Value

**Prompt**:
```
Configure credentials with environment "production" (should be "mainnet")
```

**Expected Error Response**:
```json
{
  "error_code": "INVALID_ENVIRONMENT",
  "message": "Environment must be 'testnet' or 'mainnet'"
}
```

**Validation**:
✅ Error code: `INVALID_ENVIRONMENT`
✅ Message clarifies valid values

### Test 3c: Wrong API Credentials (Async Validation)

**Setup**: Configure with valid format but wrong credentials
```
API Key: 0000000000000000000000000000000000000000000000000000000000000000
API Secret: 0000000000000000000000000000000000000000000000000000000000000000
Environment: testnet
```

**Prompt**:
```
Get my account info
```

**Expected Error Response** (from Binance API on first call):
```json
{
  "error_code": "BINANCE_API_ERROR",
  "message": "Invalid API-key, IP, or permissions for action",
  "binance_code": -2015
}
```

**Validation**:
✅ `configure_credentials` succeeds (format validation passed)
✅ First authenticated tool call fails with `BINANCE_API_ERROR`
✅ Binance error code preserved: `binance_code: -2015`

---

## Scenario 4: Revoke Credentials (P2 - Security)

**Goal**: Clear credentials mid-session for security

### Step 1: Revoke Credentials

**Prompt**:
```
Revoke my Binance credentials from this session
```

**Expected MCP Tool Call**:
```json
{
  "tool": "revoke_credentials",
  "arguments": {}
}
```

**Expected Response**:
```json
{
  "revoked": true,
  "message": "Credentials successfully revoked from session"
}
```

### Step 2: Verify Credentials Cleared

**Prompt**:
```
What are my current credential settings?
```

**Expected Response**:
```json
{
  "configured": false
}
```

**Validation**:
✅ `configured: false` (credentials cleared)
✅ No `environment`, `key_prefix`, or `configured_at` fields (only returned when configured)

### Step 3: Verify Account Tools Blocked

**Prompt**:
```
Show my account info
```

**Expected Error Response** (SC-006):
```json
{
  "error_code": "CREDENTIALS_NOT_CONFIGURED",
  "message": "API credentials not configured for this session. Call configure_credentials first."
}
```

**Success Criteria** (SC-006): Error returned within 50ms (fast-fail)

**Validation**:
✅ Error code: `CREDENTIALS_NOT_CONFIGURED`
✅ Fast response (no network request to Binance API)

---

## Scenario 5: Public Tools Work Without Credentials (FR-011)

**Goal**: Verify public data tools work without credential configuration

### Test: Get Ticker Without Credentials

**Setup**: Ensure credentials are revoked (from Scenario 4)

**Prompt**:
```
What's the current BTC/USDT price?
```

**Expected MCP Tool Call**:
```json
{
  "tool": "get_ticker",
  "arguments": {
    "symbol": "BTCUSDT"
  }
}
```

**Expected Response** (example):
```json
{
  "symbol": "BTCUSDT",
  "price": "43250.50"
}
```

**Validation**:
✅ Tool works without credentials configured
✅ Response shows current mainnet price (public tools always use mainnet per FR-011)
✅ Request went to `https://api.binance.com` (mainnet) even though no credentials configured

**Other Public Tools** (all work without credentials):
- `get_klines` (candlestick data)
- `get_order_book` (depth data)
- `get_average_price`
- `get_recent_trades`
- `search` / `fetch` (ChatGPT-specific tools)

---

## Validation Checklist

Use this checklist to verify implementation correctness:

### Core Functionality (P1)
- [ ] **SC-001**: All 7 account/trading tools return valid data when credentials configured
  - get_account_info, get_account_trades, place_order, get_order, cancel_order, get_open_orders, get_all_orders
- [ ] **SC-002**: 0% credential leakage between sessions (test with 2 concurrent ChatGPT sessions)
- [ ] **SC-004**: Credential configuration completes within 100ms
- [ ] **FR-001**: Each session has isolated credentials (Mcp-Session-Id isolation)
- [ ] **FR-003**: Credentials cleared immediately when session ends (close ChatGPT connection, reconnect, verify not configured)

### Error Handling (P1)
- [ ] **SC-006**: CREDENTIALS_NOT_CONFIGURED error returned within 50ms when unconfigured
- [ ] **SC-007**: Format validation errors detected synchronously within 10ms
- [ ] **FR-009**: All errors return structured JSON with error_code + message
- [ ] **FR-013**: All 6 error codes work correctly:
  - CREDENTIALS_NOT_CONFIGURED
  - INVALID_API_KEY_FORMAT
  - INVALID_API_SECRET_FORMAT
  - INVALID_ENVIRONMENT
  - BINANCE_API_ERROR (with binance_code field)
  - BINANCE_RATE_LIMIT (with retry_after field)

### Environment Switching (P2)
- [ ] **SC-005**: Environment switch (testnet → mainnet) takes effect within 1 API call
- [ ] **FR-002**: Testnet uses `https://testnet.binance.vision`, mainnet uses `https://api.binance.com`
- [ ] **FR-011**: Public tools always use mainnet regardless of configured environment

### Security (NON-NEGOTIABLE)
- [ ] **SC-003**: 0% credential persistence to disk (check filesystem after 1000 configure cycles)
- [ ] **NFR-002**: API secrets never logged at any log level (check server logs)
- [ ] **NFR-003**: Credential status only shows first 8 characters of API key
- [ ] **FR-004**: No credentials in disk, environment variables, or config files

### Performance (NFR)
- [ ] **NFR-004**: Zero latency overhead for public tools (get_ticker works same speed with/without credentials)

---

## Troubleshooting

### "INVALID_API_KEY_FORMAT" error
**Cause**: API key is not exactly 64 alphanumeric characters
**Solution**: Check your API key - it should look like `1234567890abcdef...` (64 chars total)

### "BINANCE_API_ERROR: Invalid API-key, IP, or permissions for action" (code -2015)
**Cause**: API key/secret is wrong, or IP not whitelisted, or missing permissions
**Solutions**:
1. Verify API key/secret copied correctly (no spaces, complete 64 chars)
2. Check Binance UI: API key has "Enable Spot & Margin Trading" permission
3. If testnet: Ensure using testnet.binance.vision credentials, not mainnet
4. If IP whitelisting enabled: Add your server's IP to whitelist in Binance UI

### "BINANCE_RATE_LIMIT" error
**Cause**: Exceeded Binance API rate limit (1200 req/min for signed endpoints)
**Solution**: Wait `retry_after` seconds (shown in error response), then retry

### Account tools work on testnet but fail on mainnet
**Cause**: Different API keys have different permissions
**Solution**: Verify mainnet API key has required permissions in Binance UI → API Management → Edit restrictions

---

## Next Steps

After completing this quickstart:

1. **Try placing a test order** on testnet:
   ```
   Place a limit buy order for 0.001 BTC at 40000 USDT on testnet
   ```

2. **Check order history**:
   ```
   Show my open orders on testnet
   ```

3. **Switch to mainnet** for real trading (use caution!)

4. **Implement automated trading strategies** via ChatGPT + MCP tools

For detailed API documentation, see:
- Testnet: https://testnet.binance.vision/
- Mainnet: https://binance-docs.github.io/apidocs/spot/en/

For implementation details, see:
- [data-model.md](./data-model.md) - Entity definitions
- [contracts/](./contracts/) - MCP tool JSON schemas
- [research.md](./research.md) - Technical design decisions
