# Tasks: Mainnet Support with Secure API Key Authentication

**Input**: Design documents from `/specs/011-mainnet-api-auth/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This feature specification does NOT explicitly request tests, so test tasks are omitted per template guidance.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- Single Rust project: `src/`, `tests/` at repository root
- Paths reference repository root: `/Users/vi/project/tradeforge/mcp-binance-rs/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependency updates

- [X] T001 Add regex dependency to Cargo.toml (version 1.11+ for format validation)
- [X] T002 [P] Add once_cell dependency to Cargo.toml (version 1.20+ for lazy static regex compilation)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core credential infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Create src/types.rs module and Environment enum with Testnet and Mainnet variants, base_url() method, from_str() method, serde serialize/deserialize
- [X] T004 Create CredentialError enum in src/error/mod.rs with 6 variants (NotConfigured, InvalidApiKeyFormat, InvalidApiSecretFormat, InvalidEnvironment, BinanceApiError, RateLimitExceeded) and to_json() method
- [X] T005 Create Credentials struct in src/transport/sse/session.rs with fields (api_key, api_secret, environment, configured_at, session_id) and key_prefix() method, importing Environment from crate::types
- [X] T006 Add credentials: Arc<RwLock<HashMap<String, Credentials>>> field to SessionManager in src/transport/sse/session.rs
- [X] T007 Implement SessionManager::store_credentials() method in src/transport/sse/session.rs
- [X] T008 [P] Implement SessionManager::get_credentials() method in src/transport/sse/session.rs
- [X] T009 [P] Implement SessionManager::revoke_credentials() method in src/transport/sse/session.rs
- [X] T010 Modify SessionManager cleanup logic in src/transport/sse/session.rs to remove credentials when session expires

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Configure API Credentials for Session (Priority: P1) 🎯 MVP

**Goal**: Enable users to configure Binance API credentials (testnet/mainnet) via MCP tools and use account/trading tools

**Independent Test**: Start new ChatGPT session → Call `configure_credentials` with API key/secret/environment → Call `get_account_info` → Verify account data returns

### Implementation for User Story 1

- [X] T011 [US1] Create src/tools/credentials.rs module file
- [X] T012 [US1] Implement validate_api_key() function in src/tools/credentials.rs using Lazy<Regex> pattern matching ^[A-Za-z0-9]{64}$
- [X] T013 [P] [US1] Implement validate_api_secret() function in src/tools/credentials.rs using Lazy<Regex> pattern matching ^[A-Za-z0-9]{64}$
- [X] T014 [US1] Implement configure_credentials tool handler in src/tools/credentials.rs (validates format, creates Credentials struct, calls SessionManager::store_credentials)
- [X] T015 [US1] Add configure_credentials tool to tool_router in src/server/tool_router.rs with #[tool] macro
- [X] T016 [US1] Refactor BinanceClient::signed_request() in src/binance/client.rs to accept Option<&Credentials> parameter instead of reading env vars
- [X] T017 [US1] Refactor BinanceClient::get_account_info() in src/binance/client.rs to accept Option<&Credentials> parameter and pass to signed_request
- [X] T018 [US1] Refactor BinanceClient::get_account_trades() in src/binance/client.rs to accept Option<&Credentials> parameter
- [X] T019 [P] [US1] Refactor BinanceClient::place_order() in src/binance/client.rs to accept Option<&Credentials> parameter
- [X] T020 [P] [US1] Refactor BinanceClient::get_order() in src/binance/client.rs to accept Option<&Credentials> parameter
- [X] T021 [P] [US1] Refactor BinanceClient::cancel_order() in src/binance/client.rs to accept Option<&Credentials> parameter
- [X] T022 [P] [US1] Refactor BinanceClient::get_open_orders() in src/binance/client.rs to accept Option<&Credentials> parameter
- [X] T023 [P] [US1] Refactor BinanceClient::get_all_orders() in src/binance/client.rs to accept Option<&Credentials> parameter
- [X] T024 [US1] Modify get_account_info tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T025 [US1] Modify get_account_trades tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T026 [P] [US1] Modify place_order tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T027 [P] [US1] Modify get_order tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T028 [P] [US1] Modify cancel_order tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T029 [P] [US1] Modify get_open_orders tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T030 [P] [US1] Modify get_all_orders tool handler in src/tools/mod.rs to retrieve session credentials and pass to BinanceClient
- [X] T031 [US1] Update all account/trading tool handlers to return CredentialError::NotConfigured when credentials not found in session

**Checkpoint**: At this point, users can configure credentials and use all account/trading tools

---

## Phase 4: User Story 2 - Secure Credential Isolation (Priority: P1)

**Goal**: Ensure credentials are session-isolated, never persisted to disk, and cleared on session end

**Independent Test**: Configure credentials in Session A → Start Session B → Verify Session B cannot access Session A's credentials → Restart server → Verify no persistence

### Implementation for User Story 2

- [ ] T032 [US2] Implement get_credentials_status tool handler in src/tools/credentials.rs (returns configured: false when no credentials, or full status with environment/key_prefix/configured_at when configured)
- [ ] T033 [US2] Add get_credentials_status tool to tool_router in src/server/tool_router.rs with #[tool] macro
- [ ] T034 [US2] Add #[serde(skip)] attribute to Credentials.session_id field in src/transport/sse/session.rs to prevent serialization
- [ ] T035 [US2] Verify SessionManager::cleanup() in src/transport/sse/session.rs removes credentials atomically with session removal
- [ ] T036 [US2] Add tracing::warn! log in SessionManager::store_credentials() when replacing existing credentials (last write wins behavior)

**Checkpoint**: Credential isolation and security guarantees are enforced

---

## Phase 5: User Story 3 - Revoke Credentials from Session (Priority: P2)

**Goal**: Enable users to clear credentials mid-session without closing connection

**Independent Test**: Configure credentials → Verify get_account_info works → Call revoke_credentials → Verify get_account_info returns CREDENTIALS_NOT_CONFIGURED error

### Implementation for User Story 3

- [ ] T037 [US3] Implement revoke_credentials tool handler in src/tools/credentials.rs (calls SessionManager::revoke_credentials, returns {revoked: true, message: "Credentials successfully revoked from session"})
- [ ] T038 [US3] Add revoke_credentials tool to tool_router in src/server/tool_router.rs with #[tool] macro

**Checkpoint**: Users can revoke credentials without closing session

---

## Phase 6: User Story 4 - Environment-Specific Tool Behavior (Priority: P2)

**Goal**: Ensure account/trading tools use correct endpoints (testnet vs mainnet) based on configured environment

**Independent Test**: Configure testnet credentials → Call place_order → Verify order goes to testnet.binance.vision → Reconfigure with mainnet → Call get_account_info → Verify request goes to api.binance.com

### Implementation for User Story 4

- [ ] T039 [US4] Update BinanceClient::signed_request() in src/binance/client.rs to use credentials.environment.base_url() when credentials provided
- [ ] T040 [US4] Add environment indicator to BinanceClient response logging in src/binance/client.rs (log which environment was used for request)
- [ ] T041 [US4] Verify public tools (get_ticker, get_klines, get_order_book, get_average_price, get_recent_trades) in src/tools/ always use mainnet regardless of configured credentials

**Checkpoint**: All user stories are now independently functional

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T042 [P] Add comprehensive error_code examples to CredentialError::to_json() documentation in src/error/mod.rs
- [ ] T043 Add inline code documentation to SessionManager credential methods in src/transport/sse/session.rs
- [ ] T044 Verify API secrets are never logged by checking tracing statements in src/tools/credentials.rs and src/binance/client.rs
- [ ] T045 [P] Add quickstart.md validation: Run Scenario 1 (Configure Testnet Credentials) manually and verify all 4 steps work
- [ ] T046 [P] Add quickstart.md validation: Run Scenario 3 (Handle Invalid Credentials) test cases 3a, 3b, 3c and verify error codes match
- [ ] T047 Update CLAUDE.md with Feature 011 completion date and credential management patterns
- [ ] T048 Code cleanup: Remove any unused imports from src/tools/credentials.rs and src/error/mod.rs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User Story 1 (P1) can start after Foundational
  - User Story 2 (P1) depends on User Story 1 completion (needs configure_credentials to exist)
  - User Story 3 (P2) depends on User Story 1 completion (needs credential storage to exist)
  - User Story 4 (P2) depends on User Story 1 completion (needs BinanceClient refactor)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Depends on US1 (needs configure_credentials tool and credential storage)
- **User Story 3 (P2)**: Depends on US1 (needs credential storage infrastructure)
- **User Story 4 (P2)**: Depends on US1 (needs BinanceClient refactor to accept credentials)

### Within Each User Story

**User Story 1**:
- T011-T013 (module creation + validation functions) → T014 (configure_credentials handler)
- T014 → T015 (add to router)
- T016 (BinanceClient refactor) → T017-T023 (refactor individual methods)
- T017-T023 complete → T024-T030 (update tool handlers)
- T024-T030 complete → T031 (add error handling)

**User Story 2**:
- T032 → T033 (get_credentials_status implementation and registration)
- T034, T035, T036 can run in parallel (independent file sections)

**User Story 3**:
- T037 → T038 (revoke_credentials implementation and registration)

**User Story 4**:
- T039 → T040 (BinanceClient logging after base_url implementation)
- T041 independent (verify public tools)

### Parallel Opportunities

- **Phase 1 Setup**: T001, T002 can run in parallel (different Cargo.toml lines)
- **Phase 2 Foundational**: T008, T009 can run in parallel (different SessionManager methods)
- **User Story 1**:
  - T012, T013 can run in parallel (different validation functions)
  - T019, T020, T021, T022, T023 can run in parallel (different BinanceClient methods)
  - T026, T027, T028, T029, T030 can run in parallel (different tool handlers)
- **User Story 2**: T034, T035, T036 can run in parallel
- **Phase 7 Polish**: T042, T045, T046 can run in parallel (different files/scenarios)

---

## Parallel Example: User Story 1

```bash
# Launch validation functions together:
Task T012: "Implement validate_api_key() in src/tools/credentials.rs"
Task T013: "Implement validate_api_secret() in src/tools/credentials.rs"

# After T016 completes, launch BinanceClient method refactors together:
Task T019: "Refactor place_order() in src/binance/client.rs"
Task T020: "Refactor get_order() in src/binance/client.rs"
Task T021: "Refactor cancel_order() in src/binance/client.rs"
Task T022: "Refactor get_open_orders() in src/binance/client.rs"
Task T023: "Refactor get_all_orders() in src/binance/client.rs"

# After T017-T023 complete, launch tool handler updates together:
Task T026: "Modify place_order tool handler in src/tools/"
Task T027: "Modify get_order tool handler in src/tools/"
Task T028: "Modify cancel_order tool handler in src/tools/"
Task T029: "Modify get_open_orders tool handler in src/tools/"
Task T030: "Modify get_all_orders tool handler in src/tools/"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T010) - **CRITICAL BLOCKER**
3. Complete Phase 3: User Story 1 (T011-T031)
4. **STOP and VALIDATE**: Test credential configuration → account info retrieval
5. User Story 1 = Complete MVP - users can configure credentials and use account/trading tools

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready (10 tasks)
2. Add User Story 1 → Test independently → Deploy/Demo (21 tasks, MVP ready!)
3. Add User Story 2 → Test independently → Deploy/Demo (5 tasks, adds security validation)
4. Add User Story 3 → Test independently → Deploy/Demo (2 tasks, adds revocation)
5. Add User Story 4 → Test independently → Deploy/Demo (3 tasks, adds environment awareness)
6. Add Polish → Final release (7 tasks)

### Sequential Strategy (Recommended)

Given user story dependencies, recommend sequential execution:

1. Phase 1: Setup (2 tasks)
2. Phase 2: Foundational (8 tasks) ← **CRITICAL BLOCKER**
3. Phase 3: User Story 1 (21 tasks) ← **MVP COMPLETE**
4. Phase 4: User Story 2 (5 tasks)
5. Phase 5: User Story 3 (2 tasks)
6. Phase 6: User Story 4 (3 tasks)
7. Phase 7: Polish (7 tasks)

**Total: 48 tasks**

---

## Notes

- [P] tasks = different files/methods, no dependencies, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently testable after completion
- Tests NOT included per feature specification (no explicit test request)
- Commit after each task or logical group (e.g., after T023 complete all BinanceClient refactors)
- Stop at any checkpoint to validate story independently
- Avoid: Logging API secrets anywhere (NFR-002), persisting credentials to disk (FR-004), breaking session isolation (FR-001)
- Format validation is synchronous (<10ms, SC-007), API validation is asynchronous (on first tool call)
- Public tools always use mainnet regardless of credentials (FR-011)
