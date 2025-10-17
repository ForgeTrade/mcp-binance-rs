# Research: MCP Enhancements - Prompts, Resources & Error Handling

**Feature**: [spec.md](spec.md)
**Created**: 2025-10-17
**Status**: Phase 0 Complete

## Technology Decisions

### Decision 1: rmcp SDK Prompt/Resource Macros

**Context**: Need to implement MCP Prompts and Resources capabilities per FR-001 to FR-016. Two approaches identified:

1. **Manual Implementation** - Implement `list_prompts()`, `get_prompt()`, `list_resources()`, `read_resource()` methods directly in ServerHandler
2. **Macro-Based Implementation** - Use rmcp v0.8.1 `#[prompt_router]` and resource handler macros

**Chosen**: Macro-Based Implementation via rmcp v0.8.1

**Rationale**:
- **Constitution Compliance**: Core Principle II (Auto-Generation Priority) mandates preferring code generation over manual implementation
- **Proven Pattern**: Counter example (`rust-sdk/examples/servers/src/common/counter.rs`) demonstrates coexistence of `#[tool_router]` and `#[prompt_router]` on same impl block
- **Type Safety**: Macros generate JSON Schema automatically via schemars, enforcing Core Principle IV (Type Safety)
- **Protocol Compliance**: Macros ensure correct MCP protocol response formats (GetPromptResult, Resource types), supporting Core Principle V
- **Reduced Boilerplate**: Eliminates manual JSON serialization, parameter extraction, and error mapping

**Trade-offs**:
- (+) Eliminates ~200 lines of manual handler boilerplate per prompt/resource
- (+) Automatic parameter validation via schemars JSON Schema
- (+) Consistent with existing tool_router implementation pattern
- (-) Macro compile errors can be cryptic (mitigated by referencing counter.rs example)
- (-) Limited to rmcp SDK patterns (acceptable given MCP compliance requirements)

**Constitution Check**:
- ✅ Core Principle II: Uses auto-generated macro code
- ✅ Core Principle IV: Type-safe Parameters<T> wrapper with schemars validation
- ✅ Core Principle V: MCP-compliant GetPromptResult and Resource types

**References**:
- `rust-sdk/examples/servers/src/common/counter.rs:45-67` - prompt_router example
- `docs/IMPROVEMENTS.md:Section 1` - Macro-based prompt implementation guidance
- rmcp v0.8.1 documentation for prompt/resource macros

---

### Decision 2: Error Enhancement Strategy

**Context**: FR-017 to FR-024 require enhanced error handling with contextual recovery suggestions. Three strategies evaluated:

1. **Replace Existing Errors** - Create new BinanceError enum, replace all existing error types
2. **Enhance Existing Errors** - Extend current error types with additional context fields
3. **Parallel Error System** - Keep existing errors, add new BinanceError for MCP-specific enrichment

**Chosen**: Parallel Error System with BinanceError enum

**Rationale**:
- **Backward Compatibility**: Existing tool implementations continue working unchanged (tools returning `Result<T, ErrorData>` unaffected)
- **Phased Implementation**: Can add error enrichment incrementally without breaking existing tools
- **Constitution Alignment**: Core Principle VII (Machine-Optimized Development) requires specifications map to specific tasks - parallel system allows independent implementation
- **Security Compliance**: Core Principle I (Security-First) mandates error messages must not expose sensitive data - new error type enforces this via dedicated mapping to MCP ErrorData

**Design**:
```rust
// New error type for Phase 1
pub enum BinanceError {
    RateLimited { retry_after: Duration, current_weight: u32, limit: u32 },
    InvalidCredentials { masked_key: String, help_url: String },
    InvalidSymbol { provided: String, format_help: String },
    InsufficientBalance { asset: String, required: String, available: String },
    ApiError(reqwest::Error), // Wrapper for existing errors
}

impl From<BinanceError> for ErrorData {
    fn from(err: BinanceError) -> Self {
        match err {
            BinanceError::RateLimited { retry_after, .. } => {
                ErrorData::new(-32001, format!("Rate limited. Retry after {:?}", retry_after))
                    .with_data(json!({ "retry_after_secs": retry_after.as_secs(), ... }))
            },
            // ... other variants with recovery suggestions
        }
    }
}
```

**Trade-offs**:
- (+) No risk to existing 13 working tools
- (+) Can implement FR-017 to FR-024 independently
- (+) Clear separation: reqwest::Error for internal use, BinanceError for MCP responses
- (-) Two error types in codebase (justified by phased implementation strategy)
- (-) Requires explicit conversion when using new errors (acceptable overhead for safety)

**Constitution Check**:
- ✅ Core Principle I: Error messages sanitized in From<BinanceError> for ErrorData (no secrets exposed)
- ✅ Core Principle IV: Type-safe error variants with structured data
- ✅ Core Principle VII: Phased implementation allows mapping to specific tasks in tasks.md

**References**:
- `src/error.rs` - Current error implementation
- Constitution § Security-First: "Error messages MUST NOT expose sensitive data"
- MCP ErrorData documentation for error code standards

---

### Decision 3: Resource URI Scheme Design

**Context**: FR-010 to FR-012 require resource URIs for market data, account balances, and open orders. Two URI schemes evaluated:

1. **Flat Scheme** - `binance://btcusdt`, `binance://balances`, `binance://open-orders`
2. **Hierarchical Scheme** - `binance://market/btcusdt`, `binance://account/balances`, `binance://orders/open`

**Chosen**: Hierarchical Scheme with category/identifier pattern

**Rationale**:
- **Future Extension**: Hierarchical structure supports Phase 2-4 additions without breaking existing URIs:
  - `binance://history/trades/btcusdt` (Phase 2: historical data)
  - `binance://futures/usds/btcusdt` (Phase 4: multi-exchange support)
  - `binance://account/positions` (Phase 3: additional account resources)
- **Semantic Clarity**: Category prefix (`market/`, `account/`, `orders/`) makes resource type obvious without documentation
- **MCP Best Practices**: modelcontextprotocol documentation recommends hierarchical URIs for resource organization
- **Collision Prevention**: `binance://ethusdt` vs `binance://market/ethusdt` - latter prevents collision with future `binance://futures/ethusdt`

**URI Patterns**:
```
binance://market/{symbol}          # 24hr ticker data (FR-010)
binance://account/balances         # All account balances (FR-011)
binance://orders/open              # Active orders (FR-012)
binance://orders/open/{symbol}     # Symbol-specific open orders (future)
binance://account/positions        # Futures positions (Phase 4)
```

**Trade-offs**:
- (+) Clean namespace separation by category
- (+) URI structure maps directly to spec.md Key Entities (Resource URI definition)
- (+) Supports lowercase symbol conventions (BTCUSDT → btcusdt) for user-friendly URIs
- (-) Slightly longer URIs than flat scheme (acceptable for clarity)
- (-) Requires URI parsing logic (mitigated by simple split on '/')

**Constitution Check**:
- ✅ Core Principle III: Modular architecture - categories map to future feature modules
- ✅ Core Principle IV: Type-safe parsing of category and identifier components
- ✅ Security § Compliance: "Resource URIs follow secure design patterns without exposing sensitive identifiers" (no API keys/secrets in URIs)

**References**:
- `spec.md:155` - Resource URI entity definition
- `modelcontextprotocol/docs/tutorials/building-mcp-with-llms.mdx` - Resource design patterns
- Constitution § Modular Architecture for future-proof design

---

## Alternative Approaches Considered

### Alt 1: WebSocket for Resources (Rejected for Phase 1)

**Description**: Implement resources as WebSocket streams for real-time updates instead of static resource reads.

**Rejection Reason**:
- **Out of Scope**: spec.md explicitly lists "WebSocket support for real-time price updates" in Out of Scope section (deferred to Phase 2)
- **Complexity vs Value**: Resources are designed for quick reads; WebSocket adds significant complexity (connection management, reconnection logic, subscription state)
- **MCP Protocol Limitation**: Current MCP protocol (2024-11-05) does not support resource subscriptions/push notifications (listed in Out of Scope)
- **Constitution Violation**: Core Principle VII requires adhering to spec boundaries; implementing WebSocket would violate machine-readable specifications

**Deferral**: Phase 2 will add WebSocket support for tools (not resources), providing real-time data via tool calls instead of resource reads.

---

### Alt 2: Caching Layer for Resources (Rejected for Phase 1)

**Description**: Add Redis/in-memory cache for resource data to reduce Binance API calls.

**Rejection Reason**:
- **Out of Scope**: spec.md lists "Caching layer for resource data" in Out of Scope (deferred to Phase 2)
- **Premature Optimization**: Success Criteria SC-003 targets 40% reduction in tool calls via resources; caching not required to meet this metric
- **Complexity**: Adds cache invalidation logic, TTL management, and potential stale data issues
- **Constitution Alignment**: Core Principle VII forbids implementing features not in spec.md

**Deferral**: Phase 2 will implement caching after measuring actual resource access patterns in production.

---

### Alt 3: Prompt Versioning (Rejected)

**Description**: Support multiple versions of trading_analysis prompt (v1, v2) for backward compatibility.

**Rejection Reason**:
- **Out of Scope**: spec.md lists "Prompt versioning or A/B testing" in Out of Scope
- **YAGNI**: No existing prompt versions to maintain; premature versioning adds complexity without value
- **MCP Protocol Gap**: MCP 2024-11-05 does not define prompt versioning semantics
- **Constitution**: Core Principle VII requires specifications to explicitly request features

**Decision**: Implement single version of each prompt per FR-002 and FR-004. Future versions can be added via new feature specs if needed.

---

## Dependencies Analysis

### Required Dependencies (Already in Project)

| Dependency | Version | Usage | Constitution Check |
|------------|---------|-------|-------------------|
| rmcp | 0.8.1 | Prompt/resource macros, MCP protocol types | ✅ Core Principle V (MCP Compliance) |
| tokio | 1.48.0 | Async runtime for API calls | ✅ Core Principle VI (Async-First) |
| reqwest | 0.12.24 | HTTP client for Binance API | ✅ Existing dependency |
| serde | 1.0.228 | JSON serialization | ✅ Core Principle IV (Type Safety) |
| serde_json | 1.0.145 | JSON value manipulation for error data | ✅ Core Principle IV |
| schemars | 1.0.4 | JSON Schema for prompt parameters | ✅ Core Principle IV (Contract Enforcement) |
| thiserror | 2.0.17 | Error type definitions | ✅ Existing error handling |

**No New Dependencies Required** - All Phase 1 functionality can be implemented with existing dependency stack, per Dependency Management § Current Dependency Standards.

---

### Internal Dependencies

| Component | Location | Usage | Risk Level |
|-----------|----------|-------|------------|
| BinanceClient | `src/binance/client.rs` | API calls for prompt data fetching | LOW - Stable, 42/42 tasks complete |
| ServerHandler | `src/server/handler.rs` | Extension with prompt_handler and resource methods | LOW - Only adds new methods, no modifications |
| tool_router | `src/server/tool_router.rs` | Coexistence pattern reference for prompt_router | NONE - Read-only reference |
| Binance types | `src/binance/types.rs` | Response types for ticker, balances, orders | LOW - Reuse existing types |

**Risk Mitigation**:
- All internal dependencies have existing test coverage (integration tests in `tests/`)
- No modifications to existing tool implementations (parallel implementation per Decision 2)
- Prompt/resource handlers are additive features (no breaking changes)

---

## Phase 1 Implementation Breakdown

### Phase 1a: Prompts Foundation (MVP)

**Goal**: Implement trading_analysis prompt (FR-001 to FR-007) + basic error enhancement (FR-017, FR-018)

**Tasks**:
1. Add `#[prompt_router]` to existing ServerHandler impl block
2. Create `TradingAnalysisArgs` struct with schemars derives
3. Implement `trading_analysis` prompt handler calling `BinanceClient::get_24hr_ticker`
4. Format ticker data as markdown GetPromptResult
5. Create BinanceError enum with RateLimited variant
6. Implement `From<BinanceError>` for ErrorData with retry_after context
7. Update ServerHandler::get_info() to include `enable_prompts()` capability

**Success Criteria**: User Story 1 acceptance scenarios pass - Claude can invoke "trading_analysis" and receive formatted market data.

**Dependencies**: None (fully independent)

---

### Phase 1b: Portfolio Risk Prompt + Complete Errors

**Goal**: Add portfolio_risk prompt (FR-004) + remaining error types (FR-019 to FR-024)

**Tasks**:
1. Create `PortfolioRiskArgs` struct (empty - no parameters)
2. Implement `portfolio_risk` prompt handler calling `BinanceClient::get_account_info`
3. Format balances (free + locked) as markdown GetPromptResult
4. Add BinanceError variants: InvalidCredentials, InvalidSymbol, InsufficientBalance
5. Implement error-specific From<BinanceError> conversions with recovery suggestions
6. Add error context helpers (mask_api_key, format_balance_error)

**Success Criteria**: User Story 2 and User Story 5 acceptance scenarios pass.

**Dependencies**: Requires Phase 1a (prompt_router foundation)

---

### Phase 1c: Resources Foundation

**Goal**: Implement resource support (FR-008 to FR-016) with market data resources

**Tasks**:
1. Implement `list_resources()` in ServerHandler returning market resource URIs
2. Implement `read_resource(uri: &str)` with URI parsing (category/identifier)
3. Create resource handler for `binance://market/{symbol}` calling get_24hr_ticker
4. Format ticker data as markdown with timestamp
5. Return ResourceNotFound error for invalid URIs
6. Update ServerHandler::get_info() to include `enable_resources()` capability

**Success Criteria**: User Story 3 acceptance scenarios pass - Claude can read `binance://market/btcusdt` resource.

**Dependencies**: Requires Phase 1a (ServerHandler capability updates)

---

### Phase 1d: Account Resources

**Goal**: Add account balance and open orders resources (FR-011, FR-012)

**Tasks**:
1. Add `binance://account/balances` to list_resources()
2. Implement balances resource handler calling get_account_info
3. Format balances as markdown table with Free/Locked columns
4. Add `binance://orders/open` to list_resources()
5. Implement open orders resource handler calling get_open_orders (no symbol filter)
6. Format orders as markdown table with Symbol/Side/Type/Price/Quantity

**Success Criteria**: User Story 4 acceptance scenarios pass - all resources listed and readable.

**Dependencies**: Requires Phase 1c (resource foundation)

---

## Constitution Compliance Checklist

### ✅ Core Principle I: Security-First
- [x] No API keys in code (prompts/resources use existing BinanceClient with env vars)
- [x] HMAC signing via existing authenticated endpoints
- [x] Rate limiting respected (reusing BinanceClient rate limit logic)
- [x] Error messages sanitized via BinanceError → ErrorData conversion (no secrets exposed)
- [x] Input validation via schemars JSON Schema on prompt parameters

### ✅ Core Principle II: Auto-Generation Priority
- [x] Using rmcp macro code generation for prompts/resources
- [x] Manual code limited to handler implementations and error mapping
- [x] No manual JSON serialization (handled by macros)

### ✅ Core Principle III: Modular Architecture
- [x] Prompts/resources use existing feature-gated modules (reuses spot trading client)
- [x] No new modules required for Phase 1
- [x] Dynamic tool registration unaffected (prompts/resources are separate capabilities)

### ✅ Core Principle IV: Type Safety & Contract Enforcement
- [x] TradingAnalysisArgs with schemars derives for JSON Schema
- [x] BinanceError enum with typed variants (Duration, u32, String fields)
- [x] Parameters<T> wrapper enforces compile-time type checking
- [x] Deserialization failures return ErrorData (not silently ignored)

### ✅ Core Principle V: MCP Protocol Compliance
- [x] Prompts return GetPromptResult with markdown messages
- [x] Resources return Resource type with markdown content
- [x] Capabilities updated via enable_prompts() and enable_resources()
- [x] Error codes follow MCP standards (-32001 for rate limits, etc.)

### ✅ Core Principle VI: Async-First Design
- [x] All prompt/resource handlers are async fn
- [x] BinanceClient API calls already async via tokio
- [x] No blocking operations in handlers

### ✅ Core Principle VII: Machine-Optimized Development
- [x] Feature added via /speckit.specify workflow
- [x] Specification has Given/When/Then scenarios (5 user stories)
- [x] Functional requirements numbered FR-001 to FR-024
- [x] Success criteria measurable SC-001 to SC-008
- [x] No [NEEDS CLARIFICATION] markers in spec.md

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Macro compilation errors | Medium | Low | Reference counter.rs example, clear error messages in plan.md |
| Resource URI parsing bugs | Low | Medium | Unit tests for URI parser, comprehensive error handling |
| Error context data leakage | Low | High | Security review of BinanceError → ErrorData conversion, test with production-like keys |
| Prompt parameter validation bypass | Low | Medium | Rely on schemars JSON Schema, integration tests with invalid params |
| Coexistence of prompt_router + tool_router | Low | Low | Proven pattern in counter.rs, macros designed for this |
| Rate limit errors in prompt data fetching | Medium | Low | Reuse existing rate limit backoff, include retry_after in error context |

**High Priority Mitigations**:
1. Security review of error message sanitization (mask API keys, no stack traces)
2. Integration tests for all 5 edge cases in spec.md
3. Validation of schemars JSON Schema generation for prompt parameters

---

## Open Questions (None Blocking)

All questions from `/speckit.clarify` phase resolved as low-impact:

1. **Observability** - Not required for Phase 1; existing tracing crate covers logging needs
2. **Scalability** - Single client pattern sufficient; concurrent requests handled by tokio
3. **Reliability SLAs** - Inherits from Binance API SLAs; no additional guarantees needed

No blocking ambiguities identified. Proceed to data model and contract generation.

---

**Phase 0 Status**: ✅ Complete - Ready for Phase 1 (Data Model + Contracts)
