# Tasks: Order Book Depth Tools

**Input**: Design documents from `/specs/007-orderbook-depth-tools/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in the feature specification, so tests are omitted per template guidelines.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- Single project structure at repository root
- Module: `src/orderbook/`
- Tests: `tests/integration/`, `tests/unit/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and feature flag configuration

- [X] T001 Add `orderbook` feature flag to Cargo.toml with dependencies (tokio-tungstenite 0.27.0, rust_decimal 1.37.2, governor 0.10.1)
- [X] T002 Create `src/orderbook/` module directory structure
- [X] T003 Create `src/orderbook/mod.rs` with module exports for types, manager, metrics, websocket, rate_limiter, tools

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Implement OrderBook type in `src/orderbook/types.rs` (symbol, bids BTreeMap, asks BTreeMap, last_update_id, timestamp)
- [X] T005 [P] Implement WebSocket client in `src/orderbook/websocket.rs` (connect to `<symbol>@depth@100ms`, exponential backoff reconnection)
- [X] T006 [P] Implement rate limiter in `src/orderbook/rate_limiter.rs` (governor GCRA, 1000 req/min, 30s queue)
- [X] T007 Implement OrderBookManager in `src/orderbook/manager.rs` (track up to 20 symbols, lazy initialization, HashMap for symbol tracking)
- [X] T008 Add REST API fallback to OrderBookManager (fetch snapshot via BinanceClient, handle staleness >5s)
- [X] T009 Add delta update processing to OrderBookManager (process WebSocket depth updates, maintain BTreeMap state)
- [X] T010 Add symbol limit enforcement to OrderBookManager (20 concurrent symbols, return error on limit)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Quick Spread Assessment (Priority: P1) 🎯 MVP

**Goal**: Enable traders to quickly assess spread and liquidity conditions using L1 aggregated metrics without consuming excessive tokens

**Independent Test**: Call `get_orderbook_metrics` with BTCUSDT, verify spread_bps, microprice, bid/ask volumes, imbalance_ratio, walls, and slippage_estimates are returned within 200ms (first request may take 2-3s for lazy initialization)

### Implementation for User Story 1

- [X] T011 [P] [US1] Implement OrderBookMetrics type in `src/orderbook/types.rs` (spread_bps, microprice, bid_volume, ask_volume, imbalance_ratio, best_bid, best_ask, walls, slippage_estimates)
- [X] T012 [P] [US1] Implement Wall type in `src/orderbook/types.rs` (price, qty, side enum)
- [X] T013 [P] [US1] Implement SlippageEstimate and SlippageEstimates types in `src/orderbook/types.rs` (target_usd, avg_price, slippage_bps, filled_qty, filled_usd)
- [X] T014 [US1] Implement spread calculation in `src/orderbook/metrics.rs` (formula: ((best_ask - best_bid) / best_bid) * 10000, accuracy within 0.01 bps)
- [X] T015 [US1] Implement microprice calculation in `src/orderbook/metrics.rs` (formula: (best_bid * ask_vol + best_ask * bid_vol) / (bid_vol + ask_vol))
- [X] T016 [US1] Implement imbalance ratio calculation in `src/orderbook/metrics.rs` (formula: bid_volume / ask_volume, sum top 20 levels)
- [X] T017 [US1] Implement walls detection in `src/orderbook/metrics.rs` (identify levels with qty > 2x median of top 20 levels)
- [X] T018 [US1] Implement VWAP-based slippage estimates in `src/orderbook/metrics.rs` (calculate for 10k, 25k, 50k USD targets, both buy/sell directions)
- [X] T019 [US1] Implement `get_orderbook_metrics` MCP tool handler in `src/orderbook/tools.rs` (accept symbol param, validate symbol, call OrderBookManager, return OrderBookMetrics)
- [X] T020 [US1] Register `get_orderbook_metrics` tool in `src/server/tool_router.rs` with JSON Schema from contracts/get_orderbook_metrics.json
- [X] T021 [US1] Add error handling for SYMBOL_NOT_FOUND, SYMBOL_LIMIT_REACHED, RATE_LIMIT_EXCEEDED, INITIALIZATION_FAILED in tools.rs
- [X] T022 [US1] Add logging for lazy initialization events (INFO level), cache refresh (DEBUG level), rate limit queue (WARN when >50%)

**Checkpoint**: At this point, User Story 1 should be fully functional - traders can get L1 metrics for quick spread assessment

---

## Phase 4: User Story 2 - Detailed Depth Analysis (Priority: P2)

**Goal**: Enable traders to inspect actual price levels beyond best bid/ask using L2-lite (20 levels) or L2-full (100 levels) with compact integer scaling

**Independent Test**: Call `get_orderbook_depth` with BTCUSDT and levels=20, verify compact integer arrays (price_scale=100, qty_scale=100000) are returned within 300ms, decode first level to verify scaling

### Implementation for User Story 2

- [X] T023 [P] [US2] Implement OrderBookDepth type in `src/orderbook/types.rs` (symbol, timestamp, price_scale=100, qty_scale=100000, bids Vec<[i64; 2]>, asks Vec<[i64; 2]>)
- [X] T024 [US2] Implement compact integer encoding in `src/orderbook/metrics.rs` (scale prices by 100, quantities by 100000, convert to i64 arrays)
- [X] T025 [US2] Add depth extraction to OrderBookManager (extract top N levels from BTreeMap, apply compact scaling)
- [X] T026 [US2] Implement `get_orderbook_depth` MCP tool handler in `src/orderbook/tools.rs` (accept symbol + levels params, validate 1-100 range, return OrderBookDepth)
- [X] T027 [US2] Register `get_orderbook_depth` tool in `src/server/tool_router.rs` with JSON Schema from contracts/get_orderbook_depth.json
- [X] T028 [US2] Add error handling for INVALID_LEVELS (1-100 validation) in tools.rs
- [X] T029 [US2] Handle low liquidity edge case (return available levels if < requested levels)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - traders can escalate from L1 metrics to L2 depth analysis

---

## Phase 5: User Story 3 - Service Health Monitoring (Priority: P3)

**Goal**: Enable users to verify order book data freshness and WebSocket connection health before making trading decisions

**Independent Test**: Call `get_orderbook_health`, verify status (ok/degraded/error), orderbook_symbols_active count (0-20), last_update_age_ms (<5000 for healthy), websocket_connected boolean are returned

### Implementation for User Story 3

- [X] T030 [P] [US3] Implement OrderBookHealth type in `src/orderbook/types.rs` (status enum ok/degraded/error, orderbook_symbols_active usize, last_update_age_ms i64, websocket_connected bool, timestamp i64, reason Option<String>)
- [X] T031 [US3] Add health status calculation to OrderBookManager (check active symbol count, WebSocket connection status, last update age across all symbols)
- [X] T032 [US3] Implement health status rules in OrderBookManager (ok: all healthy + age<5s, degraded: ≥1 active but some down or age>5s, error: all connections down)
- [X] T033 [US3] Implement `get_orderbook_health` MCP tool handler in `src/orderbook/tools.rs` (no params, query OrderBookManager health, return OrderBookHealth)
- [X] T034 [US3] Register `get_orderbook_health` tool in `src/server/tool_router.rs` with JSON Schema from contracts/get_orderbook_health.json

**Checkpoint**: All user stories should now be independently functional - complete order book depth analysis toolkit is available

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and operational readiness

- [X] T035 [P] Add integration test for WebSocket reconnection in `tests/integration/orderbook_websocket.rs` (simulate disconnect, verify exponential backoff, confirm reconnect within 30s)
- [X] T036 [P] Add integration test for rate limiter in `tests/integration/orderbook_rate_limit.rs` (send 1100 requests, verify queue behavior, confirm no 418/429 errors)
- [X] T037 [P] Add integration test for metrics calculations in `tests/integration/orderbook_metrics.rs` (verify spread accuracy within 0.01 bps, microprice within $0.01, slippage within 5%)
- [X] T038 [P] Add unit tests for OrderBook type in `tests/unit/orderbook_types.rs` (test serialization, compact integer encoding/decoding)
- [X] T039 [P] Add unit tests for OrderBookManager in `tests/unit/orderbook_manager.rs` (test symbol limit enforcement, lazy initialization, cache staleness)
- [X] T040 Add performance validation (measure L1 metrics latency, verify P95 ≤200ms when warm, L2 depth ≤300ms)
- [X] T041 Add quickstart.md validation (manually execute all scenarios from quickstart.md, verify outputs match examples)
- [X] T042 [P] Update CLAUDE.md via update-agent-context.sh script with orderbook module details
- [X] T043 Code cleanup and refactoring (remove debug prints, optimize BTreeMap operations)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Uses OrderBookManager from foundational, but US1 metrics are independent
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Uses OrderBookManager health APIs, independent of US1/US2

### Within Each User Story

- **US1**: Types (T011-T013) before calculations (T014-T018), calculations before tool handler (T019), tool handler before registration (T020)
- **US2**: OrderBookDepth type (T023) before encoding logic (T024), encoding before tool handler (T026), tool handler before registration (T027)
- **US3**: OrderBookHealth type (T030) before health calculation (T031-T032), calculation before tool handler (T033), tool handler before registration (T034)

### Parallel Opportunities

- **Phase 1**: T002 and T003 can run after T001
- **Phase 2**: T005 (WebSocket), T006 (rate limiter) can run in parallel after T004
- **US1 Types**: T011, T012, T013 can all run in parallel
- **US2**: T023 can start immediately after Phase 2
- **US3**: T030 can start immediately after Phase 2
- **Polish**: T035-T039, T042 can all run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all type definitions for User Story 1 together:
Task: "Implement OrderBookMetrics type in src/orderbook/types.rs"
Task: "Implement Wall type in src/orderbook/types.rs"
Task: "Implement SlippageEstimate and SlippageEstimates types in src/orderbook/types.rs"

# After types complete, launch all calculation functions:
# (These depend on OrderBook from foundational, but are independent of each other)
# However, they all modify metrics.rs, so run sequentially or coordinate carefully
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (3 tasks)
2. Complete Phase 2: Foundational (7 tasks) - CRITICAL - blocks all stories
3. Complete Phase 3: User Story 1 (12 tasks)
4. **STOP and VALIDATE**: Test User Story 1 independently
   - Call `get_orderbook_metrics` with BTCUSDT
   - Verify spread, microprice, walls, slippage estimates
   - Verify <200ms latency (after lazy init)
   - Verify first request completes in <3s (cold start)
5. Deploy/demo if ready - MVP delivers immediate value for spread assessment

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready (10 tasks)
2. Add User Story 1 → Test independently → Deploy/Demo (MVP! - 22 tasks total)
   - Traders can now check spreads and liquidity quickly
3. Add User Story 2 → Test independently → Deploy/Demo (29 tasks total)
   - Traders can now analyze actual depth levels for support/resistance
4. Add User Story 3 → Test independently → Deploy/Demo (34 tasks total)
   - Traders can now monitor service health before critical operations
5. Add Polish → Production-ready (43 tasks total)
   - Tests, performance validation, documentation complete

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (10 tasks)
2. Once Foundational is done:
   - Developer A: User Story 1 (12 tasks)
   - Developer B: User Story 2 (7 tasks)
   - Developer C: User Story 3 (5 tasks)
3. Stories complete and integrate independently
4. Team collaborates on Polish phase (9 tasks)

---

## Task Summary

**Total Tasks**: 43

**By Phase**:
- Phase 1 (Setup): 3 tasks
- Phase 2 (Foundational): 7 tasks
- Phase 3 (US1 - Quick Spread Assessment): 12 tasks
- Phase 4 (US2 - Detailed Depth Analysis): 7 tasks
- Phase 5 (US3 - Service Health Monitoring): 5 tasks
- Phase 6 (Polish): 9 tasks

**By User Story**:
- User Story 1 (P1): 12 tasks
- User Story 2 (P2): 7 tasks
- User Story 3 (P3): 5 tasks
- Infrastructure (Setup + Foundational): 10 tasks
- Cross-cutting (Polish): 9 tasks

**Parallel Opportunities**: 15 tasks marked [P] can run in parallel within their phase

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 = 22 tasks (User Story 1 only)

**Independent Test Criteria**:
- **US1**: `get_orderbook_metrics` returns metrics within 200ms, lazy init <3s
- **US2**: `get_orderbook_depth` returns compact integer arrays within 300ms, correct scaling
- **US3**: `get_orderbook_health` returns status with accurate symbol count and staleness

---

## Notes

- [P] tasks = different files, no dependencies within phase
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Tests in Polish phase validate all user stories together
- Progressive disclosure strategy: Most users will only need US1, escalate to US2/US3 as needed
- Token economy: US1 (15% of L2-full cost) → US2-lite (50%) → US2-full (100%)
