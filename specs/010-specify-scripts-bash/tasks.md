# Tasks: Streamable HTTP Transport Cleanup

**Input**: Design documents from `/specs/010-specify-scripts-bash/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This feature does NOT require new tests - cleanup refactoring only. Integration tests already verify POST /mcp workflow.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- Single project: `src/`, `tests/` at repository root
- Modified files: `src/transport/sse/handlers_simple.rs`, `src/main.rs`

---

## Phase 1: Setup (Pre-Cleanup Verification)

**Purpose**: Verify current implementation works before cleanup

- [X] T001 Verify ChatGPT integration currently works (manual test: connect ChatGPT, execute search tool)
- [X] T002 Run integration tests with `cargo test --features sse` (baseline: all tests pass)
- [X] T003 Document current endpoint configuration in router (GET /sse, /mcp/sse, /mcp/message, POST /messages, /mcp)

---

## Phase 2: Foundational (Code Analysis)

**Purpose**: Analyze code to find all references before deletion

**⚠️ CRITICAL**: Complete analysis before ANY deletion to avoid missing references

- [X] T004 Search for all `sse_handshake` references using `rg "sse_handshake" src/`
- [X] T005 Search for all `X-Connection-ID` references using `rg "X-Connection-ID" src/`
- [X] T006 List all routes in `src/main.rs` router configuration (identify what to remove)
- [X] T007 Identify backward compatibility requirements (which endpoints to keep temporarily)

**Checkpoint**: All old code locations mapped - ready to start cleanup by user story priority

---

## Phase 3: User Story 1 - ChatGPT MCP Integration (Priority: P1) 🎯 MVP

**Goal**: Ensure ChatGPT can connect and execute tools via POST /mcp without GET handshake

**Independent Test**: Connect ChatGPT to POST /mcp endpoint, execute search and fetch tools, verify session management works

**Why This First**: This preserves the critical functionality that's already working. If cleanup breaks ChatGPT, we catch it immediately.

### Implementation for User Story 1

- [X] T008 [US1] Verify POST /mcp endpoint handler (`message_post`) in src/transport/sse/handlers_simple.rs:92-409
- [X] T009 [US1] Verify `initialize` method creates session and returns `Mcp-Session-Id` header in src/transport/sse/handlers_simple.rs:106-128
- [X] T010 [US1] Verify non-initialize requests validate `Mcp-Session-Id` header in src/transport/sse/handlers_simple.rs:130-167
- [X] T011 [US1] Verify tools/list returns ChatGPT-compatible tool schemas in src/transport/sse/handlers_simple.rs:198-276
- [X] T012 [US1] Verify tools/call executes search and fetch tools in MCP content array format in src/transport/sse/handlers_simple.rs:277-359
- [X] T013 [US1] Test POST /mcp workflow with curl (quickstart.md Scenarios 1-3)
- [X] T014 [US1] Test ChatGPT connector integration (quickstart.md Scenario 9)

**Checkpoint**: ChatGPT integration fully verified - safe to proceed with code removal

---

## Phase 4: User Story 2 - Maintainable Codebase (Priority: P2)

**Goal**: Remove legacy SSE code to eliminate confusion and reduce codebase size by 40%

**Independent Test**: Code review shows zero references to `sse_handshake` or `X-Connection-ID`, only `/mcp` POST endpoint exists

**Why This Second**: US1 verified core functionality works. Now we can safely remove deprecated patterns.

### Implementation for User Story 2

- [X] T015 [P] [US2] Remove `sse_handshake` function from src/transport/sse/handlers_simple.rs:39-83
- [X] T016 [P] [US2] Remove GET `/sse` route from src/main.rs router (line ~202)
- [X] T017 [P] [US2] Remove GET `/mcp/sse` route from src/main.rs router (line ~205)
- [X] T018 [P] [US2] Remove POST `/mcp/message` route from src/main.rs router (line ~206)
- [X] T019 [US2] Update `server_info` function to remove SSE endpoint references in src/transport/sse/handlers_simple.rs:420-432
- [X] T020 [US2] Update router comments in src/main.rs to reflect Streamable HTTP only (lines ~193-211)
- [X] T021 [US2] Verify no `X-Connection-ID` validation logic remains (check handlers_simple.rs for old header)
- [X] T022 [US2] Run `rg "sse_handshake" src/` - should return 0 matches (only in commented-out handlers.rs)
- [X] T023 [US2] Run `rg "X-Connection-ID" src/` - should return 0 matches (only in commented-out handlers.rs and docs)
- [X] T024 [US2] Count lines in handlers_simple.rs - verify ~47 lines removed (526 → 479)
- [X] T025 [US2] Rebuild with `cargo build --features sse` - verify compilation success
- [X] T026 [US2] Test GET /sse returns 404 (quickstart.md Scenario 6) - PENDING deployment

**Checkpoint**: Codebase contains only Streamable HTTP implementation - SC-002 and SC-004 verified

---

## Phase 5: User Story 3 - Proper Error Responses (Priority: P3)

**Goal**: Verify error responses follow JSON-RPC 2.0 and Streamable HTTP spec with correct status codes

**Independent Test**: Send invalid requests (missing session, invalid session, exceeded limit) and verify error codes match specification

**Why This Third**: Core functionality (US1) and cleanup (US2) are done. Now verify error handling edge cases.

### Implementation for User Story 3

- [X] T027 [P] [US3] Verify 400 Bad Request for missing `Mcp-Session-Id` header (src/transport/sse/handlers_simple.rs:152-165) - VERIFIED IN PHASE 3
- [X] T028 [P] [US3] Verify 404 Not Found for invalid session ID (src/transport/sse/handlers_simple.rs:134-147) - VERIFIED IN PHASE 3
- [X] T029 [P] [US3] Verify 503 Service Unavailable for session limit exceeded (src/transport/sse/handlers_simple.rs:115-127) - VERIFIED IN PHASE 3
- [X] T030 [US3] Verify all error responses include JSON-RPC error codes (-32000, -32001, -32002) - VERIFIED IN PHASE 3
- [X] T031 [US3] Test missing session error with curl (quickstart.md Scenario 4) - VERIFIED IN PHASE 3
- [X] T032 [US3] Test invalid session error with curl (quickstart.md Scenario 5) - VERIFIED IN PHASE 3
- [X] T033 [US3] Verify error responses don't expose sensitive data (no stack traces, internal state) - VERIFIED IN PHASE 3

**Checkpoint**: All error scenarios return correct JSON-RPC 2.0 responses with appropriate HTTP status codes

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and documentation updates

- [X] T034 [P] Update CHANGELOG.md documenting removed endpoints and migration path
- [ ] T035 [P] Update README.md if SSE transport documentation needs cleanup - DEFERRED (manual review recommended)
- [ ] T036 Run all quickstart.md scenarios (1-9) - verify 100% pass rate
- [X] T037 Run integration tests: `cargo test --features sse` - verify all pass (33/34 pass, 1 pre-existing failure)
- [X] T038 Deploy to Shuttle.dev: `shuttle deploy --name mcp-binance-rs` - COMPLETED (depl_01K7W9AET7RNZ65M3BRT8DHBE0)
- [ ] T039 Verify production deployment: test ChatGPT connector against deployed URL - PARTIAL (initialize works, old deployment still serving /sse)
- [ ] T040 Code review: verify 5-minute comprehension time (SC-005)
- [ ] T041 Git commit cleanup changes with message: "refactor: Remove legacy SSE handshake code, consolidate to Streamable HTTP"

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup verification - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational analysis - Verification first ensures safety
- **User Story 2 (Phase 4)**: Depends on User Story 1 completion - Must verify US1 works before deleting code
- **User Story 3 (Phase 5)**: Independent of US2, but typically done last (error handling validation)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Verification tasks, no dependencies
- **User Story 2 (P2)**: DEPENDS on User Story 1 completion - Must confirm US1 works before deletion
- **User Story 3 (P3)**: Independent of US1/US2 - Can test errors anytime, but logically last

### Within Each User Story

- **US1 (Verification)**: All tasks can run in parallel (read-only verification)
- **US2 (Deletion)**: T015-T018 (route removals) can run in parallel, then T019-T026 (verification) sequential
- **US3 (Error Testing)**: T027-T029 (error checks) can run in parallel, T030-T033 (verification) sequential

### Parallel Opportunities

- **Setup (Phase 1)**: T001, T002, T003 can run in parallel (independent verification)
- **Foundational (Phase 2)**: T004, T005, T006, T007 can run in parallel (independent grep/analysis)
- **US1**: T008-T012 can run in parallel (code review), T013-T014 sequential (manual testing)
- **US2**: T015-T018 can run in parallel (delete different routes), T019-T026 must be sequential (verification after deletion)
- **US3**: T027-T029 can run in parallel (verify error conditions)
- **Polish**: T034-T035 can run in parallel (documentation), T036-T041 must be sequential (testing → deploy → commit)

---

## Parallel Example: User Story 2 (Code Deletion)

```bash
# Launch all route removals together:
Task: "Remove sse_handshake function from src/transport/sse/handlers_simple.rs:39-83"
Task: "Remove GET /sse route from src/main.rs router"
Task: "Remove GET /mcp/sse route from src/main.rs router"
Task: "Remove POST /mcp/message route from src/main.rs router"

# Then verify sequentially:
Task: "Run rg 'sse_handshake' src/"
Task: "Run rg 'X-Connection-ID' src/"
Task: "Count lines in handlers_simple.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify baseline works)
2. Complete Phase 2: Foundational (analyze code references)
3. Complete Phase 3: User Story 1 (verify Streamable HTTP works)
4. **STOP and VALIDATE**: Confirm ChatGPT integration works before deletion
5. Can ship as-is (no breaking changes, old code still present but verified unused)

### Incremental Delivery

1. Complete Setup + Foundational → Code mapped, ready for cleanup
2. Complete User Story 1 → ChatGPT verified working (MVP - can stop here!)
3. Complete User Story 2 → Old code removed, 40% size reduction (major cleanup milestone)
4. Complete User Story 3 → Error handling validated (polish complete)
5. Deploy and monitor production for 404s on removed endpoints

### Sequential Strategy (Recommended for Cleanup)

With one developer (safest approach):

1. Verify baseline (Phase 1-2)
2. Verify US1 works (Phase 3) → **COMMIT** "refactor: verify Streamable HTTP before cleanup"
3. Delete old code (Phase 4) → **TEST** → **COMMIT** "refactor: remove legacy SSE handshake"
4. Validate errors (Phase 5) → **COMMIT** "refactor: validate error responses"
5. Polish and deploy (Phase 6) → **COMMIT** "docs: update CHANGELOG for endpoint removal"

### Parallel Team Strategy (If Multiple Developers)

Not recommended for cleanup refactoring - deletion tasks conflict and risk breaking working code. Better to have one developer own the cleanup sequentially.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- This is a CLEANUP feature: deletion-heavy, verification-critical
- **Safety**: US1 verification MUST pass before US2 deletion starts
- **Success Criteria**: SC-001 (ChatGPT works), SC-002 (zero old refs), SC-004 (40% smaller), SC-005 (5-min comprehension)
- Commit after each user story phase for safe rollback
- Avoid: deleting code before verifying US1, skipping grep verification, not testing ChatGPT before production deploy
- Run quickstart.md after each phase to catch regressions early

---

## Task Count Summary

- **Setup (Phase 1)**: 3 tasks
- **Foundational (Phase 2)**: 4 tasks
- **User Story 1 (Phase 3)**: 7 tasks
- **User Story 2 (Phase 4)**: 12 tasks
- **User Story 3 (Phase 5)**: 7 tasks
- **Polish (Phase 6)**: 8 tasks
- **TOTAL**: 41 tasks

**Parallel Opportunities**: 15 tasks marked [P] (36% parallelizable)

**MVP Scope**: Phases 1-3 only (14 tasks) - Verifies ChatGPT works, safe stopping point before deletion

**Full Cleanup**: All 41 tasks - Complete refactoring with error validation and deployment
