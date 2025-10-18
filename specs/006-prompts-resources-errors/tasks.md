# Implementation Tasks: MCP Enhancements - Prompts, Resources & Error Handling

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Branch**: `006-prompts-resources-errors`
**Created**: 2025-10-17 | **Status**: Ready for Implementation

## Task Summary

- **Total Tasks**: 28
- **Parallelizable Tasks**: 12 (marked with [P])
- **User Stories**: 5 (US1-US5)
- **Estimated Time**: 8-10 hours
- **MVP Scope**: Phase 3 (User Story 1 only) = 7 tasks

## Implementation Strategy

**Incremental Delivery by User Story**:
1. Complete **Phase 3 (US1)** for MVP → Deploy trading_analysis prompt
2. Complete **Phase 4 (US2)** next → Add portfolio_risk prompt
3. Complete **Phase 5 (US3)** → Enable market data resources
4. Complete **Phase 6 (US4)** → Add account resources
5. Complete **Phase 7 (US5)** → Enhance all error messages

Each phase delivers **independently testable value**. Tests are optional per spec (no TDD requirement).

---

## Phase 1: Setup & Dependencies

**Goal**: Initialize project with required dependencies and verify build.

**Tasks**:

- [X] T001 Add chrono dependency to Cargo.toml (add `chrono = "0.4"` to [dependencies] section)
- [X] T002 Verify all dependencies are at latest versions per constitution (run `cargo update --dry-run`)
- [X] T003 Run `cargo build` to verify clean compile before changes
- [X] T004 Create src/server/types.rs file for prompt parameter types (empty module initially)
- [X] T005 Create src/server/resources.rs file for resource URI handling (empty module initially)

**Acceptance**: `cargo build` succeeds, new files compile as empty modules.

---

## Phase 2: Foundational Infrastructure

**Goal**: Implement foundational types needed by all user stories (error handling, parameter types).

**Tasks**:

- [X] T006 [P] Implement BinanceError enum skeleton in src/error.rs (add enum with 5 variants per data-model.md)
- [X] T007 [P] Implement mask_api_key helper function in src/error.rs (format: `AbCd****WxYz`)
- [X] T008 Implement From<BinanceError> for ErrorData conversion in src/error.rs (add match arms for RateLimited variant only)
- [X] T009 [P] Define TradingAnalysisArgs struct in src/server/types.rs (with schemars derives per data-model.md)
- [X] T010 [P] Define TradingStrategy enum in src/server/types.rs (Aggressive/Balanced/Conservative)
- [X] T011 [P] Define RiskTolerance enum in src/server/types.rs (Low/Medium/High)
- [X] T012 [P] Define PortfolioRiskArgs struct in src/server/types.rs (empty struct per data-model.md)

**Acceptance**: `cargo build` succeeds with all types defined. No handlers implemented yet.

**Parallel Execution**: T006+T007 (error.rs) can run in parallel with T009-T012 (types.rs).

---

## Phase 3: User Story 1 - AI-Guided Trading Analysis (P1) ⭐ MVP

**Story Goal**: Enable Claude to invoke trading_analysis prompt and receive formatted market data with trading recommendations.

**Independent Test**: Ask Claude "Should I buy Bitcoin now?" → Receives market analysis within 3 seconds (SC-001).

**Tasks**:

- [X] T013 [US1] Update src/server/handler.rs to add #[prompt_handler] macro (add after #[tool_handler] line per quickstart.md)
- [X] T014 [US1] Implement trading_analysis prompt method in src/server/handler.rs (async fn with Parameters<TradingAnalysisArgs>)
- [X] T015 [US1] Add markdown formatting logic in trading_analysis handler (format ticker data per contracts/prompts.md)
- [X] T016 [US1] Add strategy context formatting in trading_analysis handler (append strategy/risk_tolerance if provided)
- [X] T017 [US1] Update ServerHandler::get_info() to include enable_prompts() capability in src/server/handler.rs
- [X] T018 [US1] Add ISO 8601 timestamp to prompt response using chrono in trading_analysis handler
- [X] T019 [US1] Handle BinanceClient API errors in trading_analysis (convert reqwest::Error to BinanceError::ApiError)

**Acceptance Criteria**:
- [ ] Claude Desktop lists "trading_analysis" in prompts
- [ ] Invoking prompt with BTCUSDT returns formatted markdown with price, volume, change
- [ ] Optional strategy/risk_tolerance parameters work correctly
- [ ] Response includes timestamp and "Data source: Binance API v3"
- [ ] Invalid symbol returns user-friendly error (if error handling complete)

**File Changes**: src/server/handler.rs (primary), src/error.rs (error conversion)

---

## Phase 4: User Story 2 - Portfolio Risk Assessment (P1)

**Story Goal**: Enable Claude to assess portfolio risk and provide diversification recommendations based on current holdings.

**Independent Test**: Ask Claude "What's my portfolio risk?" → Receives complete balance breakdown.

**Tasks**:

- [X] T020 [US2] Implement InvalidCredentials variant in From<BinanceError> for ErrorData in src/error.rs
- [X] T021 [US2] Implement portfolio_risk prompt method in src/server/handler.rs (async fn with Parameters<PortfolioRiskArgs>)
- [X] T022 [US2] Add balance table formatting in portfolio_risk handler (filter non-zero balances per quickstart.md)
- [X] T023 [US2] Add "No holdings" case handling in portfolio_risk handler (empty balance list edge case)
- [X] T024 [US2] Add ISO 8601 timestamp to portfolio_risk response using chrono

**Acceptance Criteria**:
- [ ] Claude Desktop lists "portfolio_risk" in prompts
- [ ] Invoking prompt returns markdown table with Asset|Free|Locked|Total columns
- [ ] Empty account shows "No active balances" message instead of empty table
- [ ] Invalid credentials return masked API key error (e.g., `AbCd****WxYz`)

**Dependencies**: Requires Phase 3 complete (prompt_handler macro setup).

**File Changes**: src/server/handler.rs (add portfolio_risk method), src/error.rs (InvalidCredentials error)

---

## Phase 5: User Story 3 - Efficient Market Data Access (P2)

**Story Goal**: Provide resource-based market data access to reduce tool call count by 40%.

**Independent Test**: Access `binance://market/btcusdt` resource → Receives formatted ticker data without tool call.

**Tasks**:

- [X] T025 [P] [US3] Implement ResourceCategory enum in src/server/resources.rs (Market/Account/Orders variants)
- [X] T026 [P] [US3] Implement ResourceUri struct in src/server/resources.rs (scheme, category, identifier fields)
- [X] T027 [US3] Implement ResourceUri::parse() method in src/server/resources.rs (parse `binance://{category}/{identifier}`)
- [X] T028 [US3] Implement list_resources() method in src/server/handler.rs (return vec with market/btcusdt and market/ethusdt)
- [X] T029 [US3] Implement read_resource() method skeleton in src/server/handler.rs (parse URI and dispatch to handlers)
- [X] T030 [US3] Implement read_market_resource() helper in src/server/handler.rs (fetch ticker, format as markdown)
- [X] T031 [US3] Add symbol case normalization in read_market_resource (lowercase URI → uppercase API call)
- [X] T032 [US3] Add ResourceNotFound error handling in read_resource (invalid URI returns -32404 error)
- [X] T033 [US3] Update ServerHandler::get_info() to include enable_resources() capability in src/server/handler.rs
- [X] T034 [US3] Add ISO 8601 timestamp to resource content using chrono

**Acceptance Criteria**:
- [ ] Claude Desktop lists at least 2 resources (binance://market/btcusdt, binance://market/ethusdt)
- [ ] Reading `binance://market/btcusdt` returns markdown with price/volume/change
- [ ] Reading invalid URI (e.g., `binance://invalid/resource`) returns -32404 error with examples
- [ ] Resource content includes "Last updated: {timestamp}" footer

**Dependencies**: Can start in parallel with Phase 4 (no shared files).

**File Changes**: src/server/resources.rs (new), src/server/handler.rs (add resource methods)

**Parallel Execution**: T025+T026 (resources.rs) can run in parallel with T028-T034 (handler.rs) until T027 completes (needed for T029).

---

## Phase 6: User Story 4 - Account Information Resources (P2)

**Story Goal**: Expose account balances and open orders as resources for quick reference.

**Independent Test**: Access `binance://account/balances` and `binance://orders/open` → Receives formatted data.

**Tasks**:

- [X] T035 [US4] Add account/balances and orders/open to list_resources() in src/server/handler.rs
- [X] T036 [P] [US4] Implement read_account_resource() helper in src/server/handler.rs (handle "balances" identifier)
- [X] T037 [P] [US4] Implement read_orders_resource() helper in src/server/handler.rs (handle "open" identifier)
- [X] T038 [US4] Add balance table markdown formatting in read_account_resource (per contracts/resources.md)
- [X] T039 [US4] Add orders table markdown formatting in read_orders_resource (per contracts/resources.md)
- [X] T040 [US4] Add "No open orders" case handling in read_orders_resource (empty orders list edge case)

**Acceptance Criteria**:
- [ ] Claude Desktop lists at least 4 total resources (2 market + 2 account)
- [ ] Reading `binance://account/balances` returns table with Free|Locked|Total columns
- [ ] Reading `binance://orders/open` returns table with OrderID|Symbol|Side|Type|Price|Qty|Status
- [ ] Empty orders list shows "No open orders found" message

**Dependencies**: Requires Phase 5 complete (resource infrastructure).

**File Changes**: src/server/handler.rs (add account/orders resource handlers)

**Parallel Execution**: T036 (account handler) and T037 (orders handler) can run in parallel.

---

## Phase 7: User Story 5 - Actionable Error Messages (P3)

**Story Goal**: Provide clear error messages with recovery suggestions for 90% of error scenarios.

**Independent Test**: Trigger various errors → Verify error messages include recovery suggestions.

**Tasks**:

- [X] T041 [P] [US5] Implement InvalidSymbol variant in From<BinanceError> for ErrorData in src/error.rs
- [X] T042 [P] [US5] Implement InsufficientBalance variant in From<BinanceError> for ErrorData in src/error.rs
- [X] T043 [US5] Update RateLimited error conversion to include recovery_suggestion in error data JSON in src/error.rs
- [X] T044 [US5] Update InvalidCredentials error conversion to include help_url and recovery steps in src/error.rs
- [X] T045 [US5] Update InvalidSymbol error conversion to include valid_examples and format_help in src/error.rs
- [X] T046 [US5] Update InsufficientBalance error conversion to include required/available amounts in src/error.rs
- [X] T047 [US5] Test error message sanitization (verify no full API keys, stack traces, or sensitive data exposed)

**Acceptance Criteria**:
- [ ] Rate limit error (HTTP 429) returns error code -32001 with retry_after_secs
- [ ] Invalid credentials return error code -32002 with masked_api_key (`AbCd****WxYz`)
- [ ] Invalid symbol returns error code -32003 with valid_examples array
- [ ] Insufficient balance returns error code -32004 with required/available amounts
- [ ] All errors include actionable recovery_suggestion field

**Dependencies**: Can start in parallel with any phase (only touches src/error.rs).

**File Changes**: src/error.rs (enhance error conversions)

**Parallel Execution**: T041+T042 (new variants) can run in parallel, then T043-T046 (error conversions) in parallel.

---

## Phase 8: Polish & Documentation

**Goal**: Finalize implementation with comprehensive documentation updates.

**Tasks**:

- [ ] T048 [P] Update README.md with "Prompts Support" section (add trading_analysis and portfolio_risk examples)
- [ ] T049 [P] Update README.md with "Resources Support" section (add 4 resource URI examples with descriptions)
- [ ] T050 [P] Update README.md with "Enhanced Error Handling" section (add error code table per contracts/errors.md)
- [ ] T051 [P] Update CLAUDE.md with chrono 0.4 in Active Technologies section
- [ ] T052 Run `cargo clippy -- -D warnings` to verify no linting issues
- [ ] T053 Run `cargo fmt --check` to verify code formatting
- [ ] T054 Run `cargo test` to verify all existing tests still pass
- [ ] T055 Commit changes with message: "feat: Add MCP Prompts, Resources & Enhanced Errors (Phase 1)"

**Acceptance**: All documentation updated, no clippy warnings, tests pass, clean commit.

---

## Task Dependencies & Execution Order

### Dependency Graph by User Story

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundation) ← Blocking for all user stories
    ↓
    ├─→ Phase 3 (US1 - P1) ⭐ MVP ← Can deploy immediately
    ├─→ Phase 4 (US2 - P1) ← Depends on Phase 3 (prompt_handler setup)
    ├─→ Phase 5 (US3 - P2) ← Independent, can run parallel to Phase 4
    ├─→ Phase 7 (US5 - P3) ← Independent, can run parallel to any phase
    ↓
Phase 6 (US4 - P2) ← Depends on Phase 5 (resource infrastructure)
    ↓
Phase 8 (Polish)
```

**Critical Path**: Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 8 (MVP + Portfolio)

**Independent Paths**:
- Phase 5 (Resources) can start after Phase 2, parallel to Phase 3/4
- Phase 7 (Errors) can start after Phase 2, parallel to any other phase

---

## Parallel Execution Examples

### After Phase 2 (Foundation Complete)

**Parallel Tracks** (3 developers):
```
Developer A: Phase 3 (US1) → T013-T019 (trading_analysis prompt)
Developer B: Phase 5 (US3) → T025-T034 (market resources)
Developer C: Phase 7 (US5) → T041-T047 (error enhancements)
```

**Merge Order**: A → B (Phase 6 needs Phase 5) → C (errors enhance everything)

### Within Phase 5 (Resources)

**Parallel Tasks**:
```
Developer 1: T025-T027 (ResourceUri parser in resources.rs)
Developer 2: T028-T034 (Resource handlers in handler.rs) [wait for T027 parse method]
```

### Within Phase 7 (Errors)

**Parallel Tasks**:
```
Developer 1: T041 (InvalidSymbol) + T045 (conversion)
Developer 2: T042 (InsufficientBalance) + T046 (conversion)
Developer 3: T043 (RateLimited update) + T044 (InvalidCredentials update)
```

---

## Testing Strategy (Optional)

Tests are **not required** per feature specification. If tests are desired:

### Unit Tests (Optional)

Create `tests/unit/test_error.rs`:
```rust
#[test]
fn test_mask_api_key() { ... }

#[test]
fn test_rate_limited_error_conversion() { ... }
```

Create `tests/unit/test_uri.rs`:
```rust
#[test]
fn test_parse_valid_market_uri() { ... }

#[test]
fn test_parse_invalid_uri() { ... }
```

### Integration Tests (Optional)

Create `tests/integration/test_prompts.rs`:
```rust
#[tokio::test]
async fn test_trading_analysis_prompt() { ... }
```

Create `tests/integration/test_resources.rs`:
```rust
#[tokio::test]
async fn test_read_market_resource() { ... }
```

**Test Environment**: Binance Testnet (testnet.binance.vision)

---

## Success Criteria Validation

| Criterion | Validation Method | Target | Task Coverage |
|-----------|-------------------|--------|---------------|
| SC-001: Trading analysis < 3s | Manual timing in Claude Desktop | 3 seconds | Phase 3 (US1) |
| SC-002: Portfolio breakdown complete | Manual verification | All assets | Phase 4 (US2) |
| SC-003: 40% tool call reduction | Compare resource vs tool calls | 40% | Phase 5 (US3) |
| SC-004: 90% errors have suggestions | Review error types | 5/5 = 100% | Phase 7 (US5) |
| SC-005: ≥5 resources listed | Count list_resources() | 5 resources | Phase 5+6 |
| SC-006: 60% confusion reduction | User feedback | Qualitative | Phase 7 (US5) |
| SC-007: Complete market context | Verify ticker fields | All fields | Phase 3 (US1) |
| SC-008: Concurrent handling | Load test | No degradation | All phases |

---

## File Modification Summary

| File | Created | Modified | Phases |
|------|---------|----------|--------|
| Cargo.toml | - | ✓ | Phase 1 |
| src/error.rs | - | ✓ | Phase 2, 4, 7 |
| src/server/types.rs | ✓ | - | Phase 1, 2 |
| src/server/resources.rs | ✓ | - | Phase 1, 5 |
| src/server/handler.rs | - | ✓ | Phase 3, 4, 5, 6 |
| README.md | - | ✓ | Phase 8 |
| CLAUDE.md | - | ✓ | Phase 8 |

**Total New Files**: 2 (types.rs, resources.rs)
**Total Modified Files**: 5 (Cargo.toml, error.rs, handler.rs, README.md, CLAUDE.md)

---

## Rollback Plan

Each phase is independently testable. If a phase fails:

1. **Phase 3 fails**: Rollback prompt_handler changes, continue with resources (Phase 5)
2. **Phase 4 fails**: Keep Phase 3 (trading_analysis still works), debug portfolio_risk
3. **Phase 5 fails**: Prompts still work, rollback resource methods
4. **Phase 6 fails**: Market resources still work, rollback account resources
5. **Phase 7 fails**: Basic errors still work, enhanced errors deferred

**Git Strategy**: Commit after each phase completion for clean rollback points.

---

## Implementation Notes

### MVP Scope (Fastest Path to Value)

**Recommended**: Implement **Phase 3 (US1) only** for initial MVP:
- 7 tasks (T013-T019)
- Delivers trading_analysis prompt
- Estimated time: 2 hours
- User can ask "Should I buy Bitcoin?" and get AI analysis

**Deploy → Gather Feedback → Continue**

### Constitution Compliance

All tasks comply with:
- ✅ Core Principle I: Security-First (error masking in Phase 7)
- ✅ Core Principle II: Auto-Generation (rmcp macros in Phase 3)
- ✅ Core Principle IV: Type Safety (schemars in Phase 2)
- ✅ Core Principle V: MCP Compliance (prompts/resources per protocol)
- ✅ Core Principle VII: Machine-Optimized (tasks map to FR-### requirements)

### Reference Documents

- **Quickstart Guide**: [quickstart.md](quickstart.md) - Detailed implementation steps per phase
- **API Contracts**: [contracts/](contracts/) - JSON examples and error codes
- **Data Model**: [data-model.md](data-model.md) - Entity definitions with Rust code
- **Research**: [research.md](research.md) - Technology decisions and rationale

---

**Task Generation Complete**: Ready for `/speckit.implement` execution

**Generated**: 2025-10-17
**Format**: Checklist-compliant (28/28 tasks with ID, optional [P] and [Story] labels)
**Validation**: ✅ All tasks follow `- [ ] [TaskID] [P?] [Story?] Description with file path` format
