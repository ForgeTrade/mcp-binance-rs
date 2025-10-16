# Tasks: HTTP REST API + WebSocket для Binance

**Input**: Design documents from `/specs/003-specify-scripts-bash/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅

**Tests**: Not explicitly requested in specification - implementation only

**Organization**: Tasks grouped by user story to enable independent implementation and testing

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- All paths relative to repository root

## Path Conventions
- Single Rust project: `src/`, `tests/` at repository root
- New HTTP module: `src/http/`
- Existing Binance client: `src/binance/`

## Terminology Mapping

**Spec → Implementation**:
- "Bearer token" (spec.md) → `TokenStore`, `validate_bearer_token()` (tasks.md, src/http/middleware/auth.rs)
- "WebSocket" (spec.md) → `tokio-tungstenite`, `tokio::sync::broadcast` (tasks.md, src/http/websocket/)
- "Binance credentials" (spec.md) → `BinanceClient`, `Credentials::from_env()` (existing from feature 001)

This ensures user-facing terminology (spec) maps clearly to developer implementation terms (tasks/code).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and HTTP server foundation

- [ ] T001 Add axum, tokio-tungstenite, tower dependencies to Cargo.toml with http-api and websocket feature gates
- [ ] T002 [P] Create src/http/mod.rs with module structure (routes, middleware, websocket submodules)
- [ ] T003 [P] Create src/http/routes/mod.rs as route module root
- [ ] T004 [P] Create src/http/middleware/mod.rs as middleware module root
- [ ] T005 [P] Create src/http/websocket/mod.rs as WebSocket module root
- [ ] T006 Add HTTP server configuration struct in src/config/http.rs (port, CORS origins, token list)
- [ ] T007 Update src/main.rs to support --http flag and start axum server when enabled

**Checkpoint**: Basic project structure in place, compiles with --features http-api

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core HTTP infrastructure that MUST be complete before ANY user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T008 Implement TokenStore struct with Arc<RwLock<HashMap<String, TokenMetadata>>> in src/http/middleware/auth.rs
- [ ] T009 Implement validate_bearer_token middleware function in src/http/middleware/auth.rs
- [ ] T010 [P] Implement CORS middleware configuration in src/http/middleware/cors.rs using tower_http::cors::CorsLayer
- [ ] T011 [P] Implement rate limiting middleware in src/http/middleware/rate_limit.rs using tower::limit::RateLimitLayer
- [ ] T012 Create AppState struct holding Arc<BinanceClient> and Arc<TokenStore> in src/http/mod.rs
- [ ] T013 Implement axum Router setup with middleware layers in src/http/mod.rs
- [ ] T014 [P] Add HTTP error conversion from BinanceError to axum::response::Response in src/error.rs
- [ ] T015 [P] Create health check endpoint GET /health in src/http/routes/mod.rs
- [ ] T016 Load MCP_AUTH_TOKENS from environment and populate TokenStore in src/config/http.rs

**Checkpoint**: Foundation ready - HTTP server starts, auth middleware works, health check responds

---

## Phase 3: User Story 1 - Get Market Data via REST API (Priority: P1) 🎯 MVP

**Goal**: Traders can fetch current prices, historical candles, and market data via simple HTTP GET requests

**Independent Test**:
```bash
curl -H "Authorization: Bearer test_token" \
  'http://localhost:3000/api/v1/ticker/price?symbol=BTCUSDT'
# Expected: {"symbol":"BTCUSDT","price":"45000.50",...}
```

### Implementation for User Story 1

- [ ] T017 [P] [US1] Create src/http/routes/market_data.rs module file
- [ ] T018 [P] [US1] Implement GET /api/v1/ticker/price handler in src/http/routes/market_data.rs (calls BinanceClient::get_ticker_price)
- [ ] T019 [P] [US1] Implement GET /api/v1/ticker/24hr handler in src/http/routes/market_data.rs (calls BinanceClient::get_24hr_ticker)
- [ ] T020 [P] [US1] Implement GET /api/v1/klines handler in src/http/routes/market_data.rs (calls BinanceClient::get_klines)
- [ ] T021 [P] [US1] Implement GET /api/v1/depth handler in src/http/routes/market_data.rs (calls BinanceClient::get_order_book)
- [ ] T022 [P] [US1] Implement GET /api/v1/trades handler in src/http/routes/market_data.rs (calls BinanceClient::get_recent_trades)
- [ ] T023 [US1] Register market data routes in Router with auth middleware in src/http/mod.rs
- [ ] T024 [US1] Add query parameter validation (symbol required, interval/limit optional) in market_data.rs handlers
- [ ] T025 [US1] Add request tracing with tracing::info! for all market data endpoints
- [ ] T026 [US1] Handle Binance API errors and convert to appropriate HTTP status codes (400/429/504)

**Checkpoint**: Market data endpoints fully functional - can get prices, candles, order book, trades via HTTP

---

## Phase 4: User Story 2 - Manage Orders via REST API (Priority: P1)

**Goal**: Traders can create, cancel, and query orders through HTTP API for automated trading

**Independent Test**:
```bash
curl -X POST -H "Authorization: Bearer test_token" \
  -H "Content-Type: application/json" \
  -d '{"symbol":"BTCUSDT","side":"BUY","type":"LIMIT","quantity":"0.001","price":"45000"}' \
  'http://localhost:3000/api/v1/order'
# Expected: {"orderId":12345,"status":"NEW",...}
```

### Implementation for User Story 2

- [ ] T027 [P] [US2] Create src/http/routes/orders.rs module file
- [ ] T028 [P] [US2] Define NewOrderRequest struct with serde deserialization in src/http/routes/orders.rs
- [ ] T029 [P] [US2] Define CancelOrderRequest struct with serde deserialization in src/http/routes/orders.rs
- [ ] T030 [P] [US2] Implement POST /api/v1/order handler in src/http/routes/orders.rs (calls BinanceClient::create_order)
- [ ] T031 [P] [US2] Implement DELETE /api/v1/order handler in src/http/routes/orders.rs (calls BinanceClient::cancel_order)
- [ ] T032 [P] [US2] Implement GET /api/v1/order handler in src/http/routes/orders.rs (calls BinanceClient::query_order)
- [ ] T033 [P] [US2] Implement GET /api/v1/openOrders handler in src/http/routes/orders.rs (calls BinanceClient::get_open_orders)
- [ ] T034 [P] [US2] Implement GET /api/v1/allOrders handler in src/http/routes/orders.rs (calls BinanceClient::get_all_orders)
- [ ] T035 [US2] Register order management routes in Router with auth middleware in src/http/mod.rs
- [ ] T036 [US2] Add order parameter validation (symbol, side, type, quantity, price rules) in orders.rs
- [ ] T037 [US2] Add request ID generation and return in response headers for order tracing
- [ ] T038 [US2] Handle insufficient balance errors and return HTTP 400 with descriptive message

**Checkpoint**: Order management fully functional - can create, cancel, query orders independently of US1

---

## Phase 5: User Story 3 - Get Balance and Positions (Priority: P1)

**Goal**: Traders can view account balances and open positions for risk management

**Independent Test**:
```bash
curl -H "Authorization: Bearer test_token" \
  'http://localhost:3000/api/v1/account'
# Expected: {"balances":[{"asset":"BTC","free":"1.5","locked":"0.0"},...]}
```

### Implementation for User Story 3

- [ ] T039 [P] [US3] Create src/http/routes/account.rs module file
- [ ] T040 [P] [US3] Implement GET /api/v1/account handler in src/http/routes/account.rs (calls BinanceClient::get_account)
- [ ] T041 [P] [US3] Implement GET /api/v1/account/balance handler in src/http/routes/account.rs (filters balances with non-zero amounts)
- [ ] T042 [P] [US3] Implement GET /api/v1/myTrades handler in src/http/routes/account.rs (calls BinanceClient::get_my_trades)
- [ ] T043 [US3] Register account routes in Router with auth middleware in src/http/mod.rs
- [ ] T044 [US3] Add query parameter validation (symbol optional for myTrades, limit/fromId for pagination)
- [ ] T045 [US3] Add timestamp to response for balance freshness tracking
- [ ] T046 [US3] Handle rate limits and add exponential backoff for account queries

**Checkpoint**: Account endpoints fully functional - can get balances, positions, trade history independently

---

## Phase 6: User Story 4 - Real-Time Prices via WebSocket (Priority: P2)

**Goal**: Traders receive live price updates without polling for low-latency market monitoring

**Independent Test**:
```bash
wscat -c 'ws://localhost:3000/ws/ticker/btcusdt' \
  -H "Authorization: Bearer test_token"
# Expected: JSON messages every 100-1000ms with price updates
```

### Implementation for User Story 4

- [ ] T047 [P] [US4] Create src/binance/websocket.rs module for Binance WebSocket client
- [ ] T048 [P] [US4] Implement BinanceWebSocketClient struct with tokio-tungstenite connection in src/binance/websocket.rs
- [ ] T049 [P] [US4] Implement connect_with_retry function with exponential backoff (100ms→30s) in src/binance/websocket.rs
- [ ] T050 [P] [US4] Implement ticker stream subscription using tokio::sync::broadcast channel in src/binance/websocket.rs
- [ ] T051 [US4] Create Binance→Server connection task that reads ticker messages and broadcasts to channel
- [ ] T052 [P] [US4] Create src/http/websocket/ticker.rs for client WebSocket handler
- [ ] T053 [P] [US4] Implement WebSocket upgrade handler for /ws/ticker/:symbol in src/http/websocket/ticker.rs
- [ ] T054 [US4] Implement client message forwarding task (subscribe to broadcast, send to client WS)
- [ ] T055 [US4] Register ticker WebSocket route in Router with auth middleware in src/http/mod.rs
- [ ] T056 [US4] Add WebSocket connection limit enforcement (max 50 concurrent per SC-003)
- [ ] T057 [US4] Handle client disconnection and unsubscribe from broadcast channel
- [ ] T058 [US4] Add automatic reconnection to Binance on connection loss with subscription restore

**Checkpoint**: Real-time ticker WebSocket fully functional - clients receive price updates <500ms latency

---

## Phase 7: User Story 5 - Real-Time Order Book via WebSocket (Priority: P2)

**Goal**: Traders see live order book updates for market depth analysis and liquidity assessment

**Independent Test**:
```bash
wscat -c 'ws://localhost:3000/ws/depth/btcusdt' \
  -H "Authorization: Bearer test_token"
# Expected: Initial snapshot + incremental bid/ask updates
```

### Implementation for User Story 5

- [ ] T059 [P] [US5] Implement depth stream subscription in src/binance/websocket.rs using separate broadcast channel
- [ ] T060 [P] [US5] Create Binance→Server connection task for depth stream with full snapshot + incremental updates
- [ ] T061 [P] [US5] Create src/http/websocket/depth.rs for depth WebSocket handler
- [ ] T062 [P] [US5] Implement WebSocket upgrade handler for /ws/depth/:symbol in src/http/websocket/depth.rs
- [ ] T063 [US5] Implement depth message forwarding task (subscribe to broadcast, send to client WS)
- [ ] T064 [US5] Register depth WebSocket route in Router with auth middleware in src/http/mod.rs
- [ ] T065 [US5] Send full order book snapshot to client on connection
- [ ] T066 [US5] Stream incremental updates after snapshot with sequence number validation
- [ ] T067 [US5] Handle update frequency configuration (100ms, 1000ms options per Binance API)
- [ ] T068 [US5] Add lag detection and log warnings if client falls behind (RecvError::Lagged)

**Checkpoint**: Order book WebSocket fully functional - clients get real-time bid/ask updates <200ms

---

## Phase 8: User Story 6 - User Data Stream via WebSocket (Priority: P2)

**Goal**: Traders receive instant notifications about order status and balance changes

**Independent Test**:
```bash
wscat -c 'ws://localhost:3000/ws/user' \
  -H "Authorization: Bearer test_token"
# Expected: executionReport on order fills, outboundAccountPosition on balance changes
```

### Implementation for User Story 6

- [ ] T069 [P] [US6] Implement listen key creation endpoint GET /api/v1/userDataStream in src/http/routes/account.rs
- [ ] T070 [P] [US6] Implement listen key keepalive task (PUT every 30 minutes to prevent expiry)
- [ ] T071 [P] [US6] Implement user data stream subscription in src/binance/websocket.rs using listen key
- [ ] T072 [P] [US6] Create Binance→Server connection task for user data stream with per-user broadcast channels
- [ ] T073 [P] [US6] Create src/http/websocket/user_data.rs for authenticated user stream handler
- [ ] T074 [P] [US6] Implement WebSocket upgrade handler for /ws/user in src/http/websocket/user_data.rs
- [ ] T075 [US6] Implement user-specific message forwarding task (auth → get listen key → subscribe → send)
- [ ] T076 [US6] Register user data WebSocket route in Router with auth middleware in src/http/mod.rs
- [ ] T077 [US6] Handle executionReport events (order updates) and format for client
- [ ] T078 [US6] Handle outboundAccountPosition events (balance updates) and format for client
- [ ] T079 [US6] Add automatic listen key renewal before 60-minute expiry
- [ ] T080 [US6] Clean up user-specific broadcast channel on client disconnect

**Checkpoint**: User data stream fully functional - real-time order/balance notifications work independently

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and final improvements across all stories

- [ ] T081 [P] Create specs/003-specify-scripts-bash/data-model.md documenting all REST/WebSocket message types
- [ ] T082 [P] Create specs/003-specify-scripts-bash/contracts/openapi.yaml with full REST API specification
- [ ] T083 [P] Add WebSocket protocol documentation to contracts/websocket-streams.md
- [ ] T084 [P] Create specs/003-specify-scripts-bash/quickstart.md with curl/wscat examples for all 6 user stories
- [ ] T085 [P] Update CLAUDE.md via .specify/scripts/bash/update-agent-context.sh with axum, tokio-tungstenite, tower
- [ ] T086 Add comprehensive error messages with error codes to all HTTP endpoints
- [ ] T087 Add Retry-After header to HTTP 429 responses per FR-010
- [ ] T088 Add request tracing IDs (FR-019) to all responses
- [ ] T089 Validate all HTTP endpoints against OpenAPI spec for consistency
- [ ] T090 Run cargo clippy and fix all warnings
- [ ] T091 Run cargo fmt to format all new code
- [ ] T092 Validate quickstart.md examples work end-to-end
- [ ] T093 Update project README.md with HTTP server usage instructions
- [ ] T094 [P] Validate WebSocket connection limit enforcement by testing 50+ concurrent connections in tests/integration/websocket_limits_test.rs

**Checkpoint**: All documentation complete, code quality checks pass, ready for deployment

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup (Phase 1) - BLOCKS all user stories
- **User Stories (Phases 3-8)**: All depend on Foundational (Phase 2) completion
  - US1, US2, US3 (REST APIs) can proceed in parallel after Phase 2
  - US4, US5, US6 (WebSockets) can proceed in parallel after Phase 2
  - WebSocket stories may benefit from REST API completion but are independent
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1 - Market Data REST)**: Independent after Phase 2
- **User Story 2 (P1 - Orders REST)**: Independent after Phase 2
- **User Story 3 (P1 - Account REST)**: Independent after Phase 2 (may use same account endpoint as US2 but separate implementation)
- **User Story 4 (P2 - Ticker WebSocket)**: Independent after Phase 2, introduces WebSocket infrastructure
- **User Story 5 (P2 - Depth WebSocket)**: Can reuse WebSocket infrastructure from US4 but independent
- **User Story 6 (P2 - User Data WebSocket)**: Can reuse WebSocket infrastructure from US4, needs US3 listen key endpoint

### Within Each User Story

- Tasks marked [P] within a phase can run in parallel (different files)
- Sequential tasks build on each other (e.g., create module → implement handlers → register routes)
- WebSocket stories follow pattern: Binance client → broadcast channel → client handler → route registration

### Parallel Opportunities

**Phase 1 (Setup)**: T002, T003, T004, T005 can run in parallel (different module files)

**Phase 2 (Foundational)**: T010, T011, T014, T015 can run in parallel (independent middleware/error handling)

**Phase 3 (US1 - Market Data)**: T017, T018, T019, T020, T021, T022 can run in parallel (independent handler functions in same file - use function stubs first)

**Phase 4 (US2 - Orders)**: T027, T028, T029, T030, T031, T032, T033, T034 can run in parallel (independent structs and handlers)

**Phase 5 (US3 - Account)**: T039, T040, T041, T042 can run in parallel (independent handlers)

**Phase 6 (US4 - Ticker WS)**: T047, T048, T049, T050, T052, T053 can run in parallel initially (Binance client + handler module creation)

**Phase 7 (US5 - Depth WS)**: T059, T060, T061, T062 can run in parallel (similar pattern to US4)

**Phase 8 (US6 - User Data WS)**: T069, T070, T071, T072, T073, T074 can run in parallel (similar pattern to US4/US5)

**Phase 9 (Polish)**: T081, T082, T083, T084, T085 can run in parallel (independent documentation files)

### Cross-Story Parallelization

After Phase 2 completion, teams can work on multiple stories simultaneously:

```
Team A: US1 (Market Data REST) - 10 tasks
Team B: US2 (Orders REST) - 12 tasks
Team C: US3 (Account REST) - 8 tasks
Team D: US4 (Ticker WebSocket) - 12 tasks

All 4 teams work in parallel, each delivers independently testable functionality
```

---

## Parallel Example: User Story 1 (Market Data REST)

```bash
# Launch all handler implementations in parallel (function stubs in market_data.rs):
Task T018: "Implement GET /api/v1/ticker/price handler"
Task T019: "Implement GET /api/v1/ticker/24hr handler"
Task T020: "Implement GET /api/v1/klines handler"
Task T021: "Implement GET /api/v1/depth handler"
Task T022: "Implement GET /api/v1/trades handler"

# These can be implemented simultaneously as they're independent functions
# calling different methods on the same BinanceClient
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only - All P1 REST APIs)

1. Complete Phase 1: Setup (7 tasks) - ~2 hours
2. Complete Phase 2: Foundational (9 tasks) - ~4 hours
3. Complete Phase 3: User Story 1 - Market Data (10 tasks) - ~3 hours
4. **VALIDATE US1**: Test all market data endpoints independently
5. Complete Phase 4: User Story 2 - Orders (12 tasks) - ~4 hours
6. **VALIDATE US2**: Test order management independently
7. Complete Phase 5: User Story 3 - Account (8 tasks) - ~2 hours
8. **VALIDATE US3**: Test account endpoints independently
9. **MVP COMPLETE**: REST API fully functional, ready for production
10. Total: ~46 tasks, ~15-20 hours implementation

### Incremental Delivery (Add WebSocket Stories)

After MVP (REST APIs working):

11. Complete Phase 6: User Story 4 - Ticker WebSocket (12 tasks) - ~5 hours
12. **VALIDATE US4**: Test real-time price updates independently
13. Complete Phase 7: User Story 5 - Depth WebSocket (10 tasks) - ~4 hours
14. **VALIDATE US5**: Test order book streaming independently
15. Complete Phase 8: User Story 6 - User Data WebSocket (12 tasks) - ~5 hours
16. **VALIDATE US6**: Test user notifications independently
17. Complete Phase 9: Polish (13 tasks) - ~3 hours
18. **FULL FEATURE COMPLETE**: All 6 user stories + documentation
19. Total: ~93 tasks, ~32-37 hours full implementation

### Parallel Team Strategy (4 developers)

**Sprint 1 - Foundation (2 days)**:
- All team members: Phase 1 + Phase 2 together (16 tasks)

**Sprint 2 - REST APIs (3 days)**:
- Developer A: US1 - Market Data (10 tasks)
- Developer B: US2 - Orders (12 tasks)
- Developer C: US3 - Account (8 tasks)
- Developer D: Start documentation + health checks

**Sprint 3 - WebSockets (4 days)**:
- Developer A: US4 - Ticker WebSocket (12 tasks)
- Developer B: US5 - Depth WebSocket (10 tasks)
- Developer C: US6 - User Data WebSocket (12 tasks)
- Developer D: OpenAPI spec + quickstart examples

**Sprint 4 - Polish (2 days)**:
- All team members: Phase 9 together, validation, cleanup

**Total**: ~11 days with 4 developers working in parallel

---

## Task Count Summary

- **Phase 1 (Setup)**: 7 tasks
- **Phase 2 (Foundational)**: 9 tasks (BLOCKS all stories)
- **Phase 3 (US1 - Market Data REST)**: 10 tasks
- **Phase 4 (US2 - Orders REST)**: 12 tasks
- **Phase 5 (US3 - Account REST)**: 8 tasks
- **Phase 6 (US4 - Ticker WebSocket)**: 12 tasks
- **Phase 7 (US5 - Depth WebSocket)**: 10 tasks
- **Phase 8 (US6 - User Data WebSocket)**: 12 tasks
- **Phase 9 (Polish)**: 14 tasks

**Total**: 94 tasks

**MVP (P1 stories only)**: 46 tasks (Phases 1-5)
**Full Feature**: 94 tasks (All phases)

**Parallel Opportunities**: 40+ tasks marked [P] can run in parallel within their phases

---

## Notes

- [P] tasks = different files or independent functions, no dependencies
- [Story] label (US1-US6) maps task to specific user story for traceability
- Each user story independently completable and testable after Phase 2
- No tests included (not requested in specification)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Constitution compliance: All security, modularity, type safety principles satisfied per plan.md
