# Feature Specification: MCP Enhancements - Prompts, Resources & Error Handling

**Feature Branch**: `006-prompts-resources-errors`
**Created**: 2025-10-17
**Status**: Draft
**Input**: User description: "Add MCP Prompts support for AI-guided trading analysis, Resources support for efficient market data access, and enhanced error handling with contextual recovery suggestions"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - AI-Guided Trading Analysis (Priority: P1)

As a Claude Desktop user, I want to ask Claude to analyze market conditions for a specific cryptocurrency and receive actionable trading recommendations based on real-time market data.

**Why this priority**: This is the most valuable enhancement - it enables natural language interaction with market data and AI-powered analysis, which is the core value proposition of an MCP server for trading.

**Independent Test**: Can be fully tested by asking Claude "Analyze BTCUSDT for a balanced trading strategy" and receiving a contextual analysis with market data and recommendations. Delivers immediate value without requiring other features.

**Acceptance Scenarios**:

1. **Given** a user asks Claude "Should I buy Bitcoin now?" via prompt, **When** Claude invokes the trading_analysis prompt, **Then** the prompt returns formatted market data (price, 24h change, volume, high/low) with context for AI analysis
2. **Given** a user specifies aggressive strategy preference, **When** Claude invokes trading_analysis prompt with strategy parameter, **Then** the prompt includes strategy preference in the analysis context
3. **Given** market data is successfully retrieved, **When** Claude processes the prompt response, **Then** Claude provides actionable trading recommendation based on current conditions

---

### User Story 2 - Portfolio Risk Assessment (Priority: P1)

As a Claude Desktop user, I want to ask Claude to assess the risk of my current cryptocurrency portfolio and get diversification recommendations.

**Why this priority**: Equally critical as trading analysis - provides portfolio-level insights that complement individual trading decisions. Essential for risk management.

**Independent Test**: Can be tested by asking Claude "What's my portfolio risk?" and receiving a breakdown of current holdings with risk assessment. Delivers value independently.

**Acceptance Scenarios**:

1. **Given** a user has active balances in their account, **When** Claude invokes the portfolio_risk prompt, **Then** the prompt returns all non-zero balances formatted for AI analysis
2. **Given** account information is retrieved, **When** the prompt is processed, **Then** Claude receives both free and locked balances for each asset
3. **Given** portfolio data is presented, **When** Claude analyzes it, **Then** Claude provides risk assessment and diversification recommendations

---

### User Story 3 - Efficient Market Data Access via Resources (Priority: P2)

As a Claude Desktop user, I want Claude to access frequently queried market data (like Bitcoin price) directly as a resource rather than making repeated tool calls, improving response speed.

**Why this priority**: Performance enhancement that reduces latency and API calls. Important but not critical for core functionality.

**Independent Test**: Can be tested by checking if Claude can access `binance://market/btcusdt` resource and receive formatted market data. Delivers caching benefits independently.

**Acceptance Scenarios**:

1. **Given** a user asks about Bitcoin, **When** Claude accesses the `binance://market/btcusdt` resource, **Then** Claude receives markdown-formatted market data without needing a tool call
2. **Given** the resource is accessed, **When** Claude reads the resource content, **Then** the response includes current price, 24h change, volume, and high/low values
3. **Given** multiple questions about the same symbol, **When** Claude accesses the resource repeatedly, **Then** each access returns fresh data efficiently

---

### User Story 4 - Account Information as Resources (Priority: P2)

As a Claude Desktop user, I want Claude to access my account balances and open orders as resources for quick reference during conversation.

**Why this priority**: Convenience feature that complements trading workflows. Enhances UX but not critical for core value.

**Independent Test**: Can be tested by checking if Claude can list and read `binance://account/balances` and `binance://orders/open` resources.

**Acceptance Scenarios**:

1. **Given** a user asks about their balances, **When** Claude accesses `binance://account/balances` resource, **Then** Claude receives formatted balance information
2. **Given** a user has open orders, **When** Claude accesses `binance://orders/open` resource, **Then** Claude receives list of active orders
3. **Given** resource URIs are available, **When** Claude lists resources, **Then** all relevant market and account resources appear in the list

---

### User Story 5 - Actionable Error Messages (Priority: P3)

As a Claude Desktop user, when something goes wrong (like invalid API keys or rate limits), I want to receive clear error messages that tell me exactly how to fix the problem.

**Why this priority**: Quality of life improvement. Improves user experience but doesn't add new functionality. Can be implemented last.

**Independent Test**: Can be tested by triggering various error conditions and verifying error messages include recovery suggestions.

**Acceptance Scenarios**:

1. **Given** invalid API credentials are configured, **When** a tool is called, **Then** the error message indicates credential issue and suggests checking environment variables
2. **Given** a rate limit is exceeded, **When** an API call fails with 429 status, **Then** the error message includes wait time and suggests reducing request frequency
3. **Given** an invalid symbol is provided (e.g., "INVALID"), **When** a market data tool is called, **Then** the error message suggests valid format and points to symbol documentation
4. **Given** insufficient balance for an order, **When** place_order is called, **Then** the error message shows required vs available balance with clear explanation

---

### Edge Cases

- What happens when Binance API is temporarily unavailable during a prompt invocation?
  - Prompts should fail gracefully with error context, allowing Claude to suggest retry

- What happens when a resource URI is accessed for a symbol with no recent trading activity?
  - Resource should return available data with indication of staleness/low volume

- What happens when rate limits are hit during prompt data gathering?
  - Prompt should include rate limit error with retry suggestion instead of partial data

- What happens when user has zero balances across all assets?
  - Portfolio risk prompt should indicate "no holdings" rather than failing

- What happens when resource URI format is incorrect (e.g., `binance://market/INVALID-SYMBOL`)?
  - Resource read should return clear 404-style error with valid URI examples

## Requirements *(mandatory)*

### Functional Requirements

#### MCP Prompts Support

- **FR-001**: System MUST implement `#[prompt_router]` macro to register prompt handlers
- **FR-002**: System MUST provide a `trading_analysis` prompt that accepts symbol, strategy preference (optional), and risk tolerance (optional) parameters
- **FR-003**: The `trading_analysis` prompt MUST fetch real-time 24h ticker data from Binance API and format it for AI analysis
- **FR-004**: System MUST provide a `portfolio_risk` prompt that retrieves account balances and formats them for AI risk assessment
- **FR-005**: Prompts MUST return `GetPromptResult` with formatted markdown messages suitable for LLM consumption
- **FR-006**: System MUST update ServerHandler capabilities to include `enable_prompts()`
- **FR-007**: Prompt implementations MUST handle API errors gracefully and include error context in prompt responses

#### MCP Resources Support

- **FR-008**: System MUST implement `list_resources()` method in ServerHandler to expose available resources
- **FR-009**: System MUST implement `read_resource()` method in ServerHandler to serve resource content
- **FR-010**: System MUST expose market data resources with URI format `binance://market/{symbol}` (e.g., `binance://market/btcusdt`)
- **FR-011**: System MUST expose account balance resource at URI `binance://account/balances`
- **FR-012**: System MUST expose open orders resource at URI `binance://orders/open`
- **FR-013**: Resource content MUST be formatted as markdown for optimal Claude Desktop display
- **FR-014**: Resources MUST include timestamp of last update in their content
- **FR-015**: System MUST return `ResourceNotFound` error for invalid or non-existent resource URIs
- **FR-016**: System MUST update ServerHandler capabilities to include `enable_resources()`

#### Enhanced Error Handling

- **FR-017**: System MUST create custom `BinanceError` enum with variants for common error scenarios (RateLimited, InvalidCredentials, InvalidSymbol, InsufficientBalance)
- **FR-018**: Rate limit errors MUST include retry_after duration, current weight, and weight limit in error data
- **FR-019**: Invalid credential errors MUST include suggestion to check environment variables and link to testnet documentation
- **FR-020**: Invalid symbol errors MUST include the invalid symbol provided and suggest valid format with examples
- **FR-021**: Insufficient balance errors MUST include asset name, required amount, and available amount
- **FR-022**: All custom errors MUST implement conversion to MCP `ErrorData` type with appropriate error codes
- **FR-023**: Error messages MUST be user-friendly and actionable, avoiding technical jargon where possible
- **FR-024**: Error data MUST include JSON context with recovery suggestions where applicable

### Key Entities

- **Prompt Definition**: Represents a registered AI prompt with name, description, arguments schema, and handler function. Contains metadata for MCP protocol and parameters for data fetching.

- **Resource Definition**: Represents an accessible data endpoint with URI, name/description, and content format. Includes logic for fetching and formatting real-time data.

- **Error Context**: Represents enriched error information including error type, user message, recovery suggestions, and structured debug data for troubleshooting.

- **Trading Analysis Arguments**: Parameters for trading analysis prompt including symbol (required), strategy preference (optional: aggressive/balanced/conservative), and risk tolerance (optional: low/medium/high).

- **Resource URI**: Hierarchical identifier for resources following pattern `binance://{category}/{identifier}` where category is "market", "account", or "orders".

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can ask Claude natural language questions like "Should I buy Bitcoin?" and receive contextual trading analysis within 3 seconds (including API latency)

- **SC-002**: Users can ask Claude "What's my portfolio risk?" and receive complete portfolio breakdown with risk assessment based on current balances

- **SC-003**: Resource access for frequently queried market data (BTCUSDT, ETHUSDT) reduces tool call count by 40% compared to tool-only implementation

- **SC-004**: 90% of error scenarios (invalid credentials, rate limits, invalid symbols, insufficient balance) provide actionable recovery suggestions in error messages

- **SC-005**: Claude Desktop lists at least 5 resources (market/btcusdt, market/ethusdt, account/balances, account/positions, orders/open) when MCP server connects

- **SC-006**: Error messages for common issues (rate limit, invalid credentials) result in 60% reduction in user confusion based on need for additional clarification

- **SC-007**: Prompt responses include all necessary market context (price, volume, change, high/low) formatted for optimal LLM analysis without requiring follow-up tool calls

- **SC-008**: System handles prompt and resource requests concurrently with existing tool calls without degradation in response time


## Assumptions

1. **Development Environment**: Developers have access to rmcp examples in `/rust-sdk/examples/servers/` for reference patterns
2. **MCP Protocol**: Current rmcp SDK (v0.8.1) supports prompts and resources features via macros
3. **API Availability**: Binance API endpoints remain stable and available at current URLs
4. **Claude Desktop**: Client supports MCP protocol version 2024-11-05 with prompts and resources
5. **Performance**: Additional prompt/resource logic does not significantly impact existing tool performance
6. **Testing**: Integration test framework can simulate prompt invocations and resource requests
7. **Error Standards**: MCP protocol defines standard error codes that map cleanly to Binance API errors
8. **Markdown Support**: Claude Desktop renders markdown content from resources correctly
9. **Concurrent Requests**: rmcp SDK handles concurrent prompt, resource, and tool requests safely
10. **API Rate Limits**: Current rate limit strategy (weight tracking, backoff) remains effective with additional endpoints

## Dependencies

1. **External Dependencies**:
   - rmcp crate v0.8.1+ with prompts and resources support
   - Existing BinanceClient implementation for API calls
   - schemars crate for JSON Schema generation of prompt parameters

2. **Internal Dependencies**:
   - Existing tool_router implementation (must coexist with prompt_router)
   - Existing ServerHandler implementation (must be extended with prompt_handler and resource methods)
   - Existing error types (will be enhanced but not replaced)
   - Existing integration test infrastructure

3. **Documentation Dependencies**:
   - Counter example at `/rust-sdk/examples/servers/src/common/counter.rs` for prompt_router pattern
   - MCP specification for resources API design
   - IMPROVEMENTS.md document with detailed implementation examples

## Out of Scope

- WebSocket support for real-time price updates (deferred to Phase 2)
- Caching layer for resource data (deferred to Phase 2)
- Progress reporting for long-running operations (deferred to Phase 2)
- Metrics collection and monitoring (deferred to Phase 3)
- Configuration hot-reload (deferred to Phase 4)
- Multi-exchange support (deferred to Phase 4)
- Resource templates with dynamic parameters
- Prompt versioning or A/B testing
- Error telemetry or automated error reporting
- Resource subscription/push notifications
- Internationalization of error messages

## Security & Compliance

### Security Considerations

1. **Credential Exposure**: Error messages MUST NOT include actual API keys or secrets in error data, only masked references (first 4 and last 4 characters)
2. **Resource Access Control**: Resources MUST require same API credentials as tools - no unauthenticated resource access
3. **Error Information Leakage**: Error messages MUST NOT expose internal system paths, stack traces, or sensitive API details
4. **Rate Limit Bypass**: Error handling MUST NOT encourage users to bypass rate limits through retry suggestions

### Compliance

- No PII (Personally Identifiable Information) stored or logged in error context
- Error messages comply with Binance API terms of service regarding error handling
- Resource URIs follow secure design patterns without exposing sensitive identifiers

## Notes & Clarifications

### Design Decisions

1. **Prompt vs Tool Choice**: Prompts are used for scenarios requiring AI analysis (trading analysis, portfolio risk) where the server provides data context for Claude to process. Tools remain for direct actions (placing orders, querying specific data).

2. **Resource URI Scheme**: Using `binance://` scheme for clear namespace separation. Pattern `{category}/{identifier}` allows for future extension (e.g., `binance://history/trades/btcusdt`).

3. **Error Enhancement Strategy**: Enhancing rather than replacing existing error handling to maintain backward compatibility with current tool implementations.

4. **Markdown Formatting**: Resources use markdown for better readability in Claude Desktop, following best practices from MCP examples.

### Implementation Priorities

1. **Phase 1a** (MVP): Implement trading_analysis prompt + basic error enhancement
2. **Phase 1b**: Add portfolio_risk prompt + remaining error types
3. **Phase 1c**: Implement resources support with market data resources
4. **Phase 1d**: Add account resources (balances, orders)

### Future Enhancements (Outside This Feature)

- Advanced prompt templates with historical data analysis
- Resource subscriptions for real-time updates
- Prompt chaining for multi-step analysis workflows
- Error analytics dashboard
