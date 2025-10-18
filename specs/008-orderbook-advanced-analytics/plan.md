# Implementation Plan: Advanced Order Book Analytics

**Branch**: `008-orderbook-advanced-analytics` | **Date**: 2025-01-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-orderbook-advanced-analytics/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

This feature extends the existing orderbook infrastructure (Feature 007) with advanced analytics capabilities: order flow analysis (bid/ask pressure tracking), volume profile generation (POC/VAH/VAL support/resistance zones), and market microstructure anomaly detection (quote stuffing, iceberg orders, flash crash precursors). Technical approach uses time-series storage for 1-second orderbook snapshots (7-day retention, ~12M snapshots for 20 pairs), aggregated trade stream (@aggTrade) for volume data, and statistical threshold-based detection algorithms. New feature gate `orderbook_analytics` separates from base `orderbook` module.

## Technical Context

**Language/Version**: Rust 1.90+ (Edition 2024)
**Primary Dependencies**:
- rmcp 0.8.1 (MCP SDK)
- tokio 1.48.0 (async runtime with WebSocket support)
- reqwest 0.12.24 (HTTP client for @aggTrade WebSocket)
- serde 1.0.228 + serde_json 1.0.145 (serialization)
- rocksdb 0.23.0 (embedded time-series storage, <1ms queries, Zstd compression) ✅ Resolved in Phase 0
- statrs 0.18.0 (statistical analysis: median, percentiles, std dev, rolling windows) ✅ Resolved in Phase 0
- tokio-tungstenite 0.24.0 (@aggTrade WebSocket client)
- rmp-serde 1.3.0 (MessagePack serialization for snapshots)

**Storage**: RocksDB embedded time-series database (1-second snapshots, 7-day retention, ~12M snapshots for 20 pairs, ~500MB-1GB with Zstd compression). Key design: `{symbol}:{unix_timestamp_sec}`, prefix scans for time-range queries. Background cleanup task deletes keys older than 7 days.

**Testing**: cargo test with mock WebSocket streams and deterministic snapshot data. Backtesting validation for anomaly detection precision (SC-003: >95% for quote stuffing, SC-004: >80% for icebergs)

**Target Platform**: Linux/macOS server (stdio + HTTP transports), no WASM (WebSocket dependency)

**Project Type**: Single Rust project with feature-gated module (`orderbook_analytics` extends base `orderbook`)

**Performance Goals**:
- Order flow calculations: <100ms per WebSocket update (NFR)
- Volume profile generation: <500ms for 24h data on BTCUSDT/ETHUSDT (SC-002)
- Anomaly detection: >1000 orderbook updates/second without dropping metrics (SC-007)
- Historical snapshot queries: <200ms (clarification Q5)

**Constraints**:
- Memory: ~50MB base + time-series storage overhead (500MB-1GB for snapshots)
- Latency: <300ms end-to-end (100ms order flow + 200ms snapshot query)
- Concurrency: 20 trading pairs with independent analytics pipelines

**Scale/Scope**:
- 20 concurrent trading pairs (parity with Feature 007 orderbook limit)
- 12M snapshots (7 days × 20 pairs × 86,400 snapshots/day)
- 5 new MCP tools: get_order_flow, get_volume_profile, detect_market_anomalies, get_liquidity_vacuums, get_microstructure_health

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First (NON-NEGOTIABLE)
- ✅ **PASS**: No new authentication requirements (uses existing Binance connection from Feature 007)
- ✅ **PASS**: No API keys stored in code (analytics uses existing credential management)
- ✅ **PASS**: Input validation on all tool parameters (symbol validation, time window bounds)
- ✅ **PASS**: Error messages do not expose sensitive data (orderbook snapshots contain no credentials)
- ✅ **PASS**: Rate limiting inherited from Feature 007 WebSocket infrastructure

### II. Auto-Generation Priority
- ✅ **PASS**: No code generation required (manual analytics implementation per spec line 179)
- ✅ **PASS**: Statistical algorithms hand-written (no authoritative schema to generate from)
- ℹ️ **N/A**: Binance does not provide OpenAPI spec for orderbook analytics (domain-specific calculations)

### III. Modular Architecture
- ✅ **PASS**: New feature gate `orderbook_analytics` separate from base `orderbook` (spec line 180)
- ✅ **PASS**: Default features remain `spot` + `server` + `transport-stdio` (no new defaults)
- ✅ **PASS**: Cross-module dependency: `orderbook_analytics` requires `orderbook` feature
- ✅ **PASS**: MCP dynamic tool registration based on enabled features

### IV. Type Safety & Contract Enforcement
- ✅ **PASS**: All metrics use strong typing (OrderFlowSnapshot, VolumeProfile, MarketMicrostructureAnomaly per spec lines 106-113)
- ✅ **PASS**: JSON Schema generation for all 5 new MCP tools via schemars
- ✅ **PASS**: Timestamp validation (time windows must be positive, end > start)
- ✅ **PASS**: Enum-based anomaly types (QuoteStuffing, IcebergOrder, FlashCrashRisk)

### V. MCP Protocol Compliance (NON-NEGOTIABLE)
- ✅ **PASS**: Full MCP lifecycle compliance (tools/list, tools/call per spec line 182)
- ✅ **PASS**: JSON Schema for all 5 new tools (spec line 182)
- ✅ **PASS**: Structured error responses (isError: true for failures)
- ✅ **PASS**: Progress notifications for long-running volume profile (>2 seconds for 24h data)

### VI. Async-First Design
- ✅ **PASS**: All calculations async and non-blocking on WebSocket thread (spec line 183)
- ✅ **PASS**: Tokio async runtime throughout (tokio 1.48.0)
- ✅ **PASS**: WebSocket streams use async channels for orderbook + @aggTrade
- ✅ **PASS**: Time-series queries are async with timeout handling

### VII. Machine-Optimized Development
- ✅ **PASS**: Feature added via `/speckit.specify` workflow (spec line 184)
- ✅ **PASS**: Machine-readable requirements (FR-001 to FR-010, SC-001 to SC-008)
- ✅ **PASS**: Given/When/Then acceptance scenarios (spec lines 20-60)
- ✅ **PASS**: All NEEDS CLARIFICATION items resolved in Phase 0 research (RocksDB 0.23.0, statrs 0.18.0)

### Dependency Management
- ✅ **PASS**: All existing dependencies up-to-date (rmcp 0.8.1, tokio 1.48.0, etc. per constitution)
- ✅ **PASS**: New dependencies verified latest stable: rocksdb 0.23.0, statrs 0.18.0, tokio-tungstenite 0.24.0, rmp-serde 1.3.0
- ✅ **PASS**: Rust Edition 2024 maintained

**GATE STATUS**: ✅ **PASS** (all clarifications resolved, ready for Phase 1 design)

## Project Structure

### Documentation (this feature)

```
specs/008-orderbook-advanced-analytics/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── get_order_flow.json
│   ├── get_volume_profile.json
│   ├── detect_market_anomalies.json
│   ├── get_liquidity_vacuums.json
│   └── get_microstructure_health.json
├── checklists/
│   └── requirements.md  # Specification quality validation (completed)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

**Structure Decision**: Single Rust project with feature-gated module. Extends existing `src/orderbook/` module (Feature 007) with new analytics submodule activated by `orderbook_analytics` cargo feature.

```
src/
├── lib.rs                          # Feature gate registration
├── server.rs                       # MCP server (adds 5 new tools when orderbook_analytics enabled)
├── orderbook/                      # Existing Feature 007 module
│   ├── mod.rs                      # Re-exports analytics when feature enabled
│   ├── types.rs                    # OrderBook, OrderBookMetrics (existing)
│   ├── websocket.rs                # WebSocket orderbook stream (existing)
│   ├── tools.rs                    # MCP tools for basic orderbook (existing)
│   └── analytics/                  # NEW: Advanced analytics submodule
│       ├── mod.rs                  # Public API + feature gate checks
│       ├── types.rs                # OrderFlowSnapshot, VolumeProfile, MarketMicrostructureAnomaly, etc.
│       ├── storage/                # Time-series snapshot storage
│       │   ├── mod.rs
│       │   ├── snapshot.rs         # 1-second snapshot capture logic
│       │   └── query.rs            # Historical data retrieval (<200ms target)
│       ├── flow.rs                 # Order flow analysis (FR-001, FR-006, FR-007)
│       ├── profile.rs              # Volume profile generation (FR-002, FR-008)
│       ├── anomaly.rs              # Microstructure anomaly detection (FR-003, FR-004, FR-005)
│       ├── health.rs               # Microstructure health scoring (FR-010)
│       ├── trade_stream.rs         # @aggTrade WebSocket integration
│       └── tools.rs                # MCP tool implementations (5 new tools)
└── config.rs                       # Environment configuration (existing)

tests/
├── orderbook_analytics_flow.rs          # Order flow calculation tests
├── orderbook_analytics_profile.rs       # Volume profile tests (POC/VAH/VAL validation)
├── orderbook_analytics_anomaly.rs       # Anomaly detection precision tests
└── orderbook_analytics_integration.rs   # End-to-end MCP tool tests
```

**Key Points**:
- Analytics module is isolated under `src/orderbook/analytics/` (clear separation from base orderbook)
- Feature gate `orderbook_analytics` in `Cargo.toml` controls compilation
- Time-series storage abstracted in `storage/` submodule (implementation choice resolved in Phase 0)
- Each analytics capability (flow, profile, anomaly, health) in separate modules for testability

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

**No violations detected** - All 7 constitution principles pass. Feature complies with security, modularity, type safety, MCP protocol, async design, and machine-optimized development requirements.

