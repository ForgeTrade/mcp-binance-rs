# Implementation Plan: Order Book Depth Tools

**Branch**: `007-orderbook-depth-tools` | **Date**: 2025-10-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-orderbook-depth-tools/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add order book depth analysis tools to MCP server with progressive disclosure strategy (L1→L2-lite→L2-full) for optimal token economy. Implements three MCP tools: `get_orderbook_metrics` (L1 aggregated metrics), `get_orderbook_depth` (L2 depth data with compact integer scaling), and `get_orderbook_health` (service health monitoring). Uses WebSocket + Local L2 Cache architecture for sub-100ms latency with REST API fallback. Supports up to 20 concurrent symbols with client-side rate limiting (1000 req/min) and lazy initialization on first request.

## Technical Context

**Language/Version**: Rust 1.90+ (Edition 2024)
**Primary Dependencies**:
- rmcp 0.8.1 (MCP Server SDK with macros)
- tokio 1.48.0 (async runtime with full features)
- tokio-tungstenite 0.27.0 (WebSocket client)
- reqwest 0.12.24 (HTTP client, json + rustls-tls)
- serde 1.0.228 + serde_json 1.0.145 (serialization)
- schemars 1.0.4 (JSON Schema generation)
- rust_decimal 1.37.2 (precision arithmetic)
- governor 0.10.1 (GCRA rate limiting)

**Storage**: In-memory (BTreeMap<Decimal, Decimal> for order book state, HashMap for symbol tracking)
**Testing**: cargo test (unit, integration tests for WebSocket reconnection, rate limiting, metrics calculations)
**Target Platform**: Server (Linux/macOS/Windows), MCP stdio transport
**Project Type**: Single project (MCP server extension)
**Performance Goals**:
- L1 metrics: P95 latency ≤200ms (WebSocket connected)
- L2 depth: P95 latency ≤300ms (both L2-lite/L2-full)
- WebSocket staleness: <5s 99.9% of the time
- First request (lazy init): ≤3s for 95% of cases

**Constraints**:
- 20 concurrent symbol limit (enforced)
- 1000 REST requests/minute rate limit (client-side)
- 30s request queue timeout before rejection
- Compact integer scaling: price_scale=100, qty_scale=100000

**Scale/Scope**:
- 20 concurrent symbols max
- 100 order book levels per symbol max
- 3 MCP tools
- 27 functional requirements
- WebSocket + REST API integration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First ✅ PASS

- API keys from environment (existing BinanceClient uses Credentials::from_env)
- Rate limiting enforced (FR-023, FR-024: 1000 req/min + 30s queue)
- Input validation (FR-012: symbol validation against Binance spot pairs)
- Error messages sanitized (no sensitive data exposure)
- No new external dependencies requiring security audit beyond existing reqwest/tokio

### II. Auto-Generation Priority ✅ PASS

- No code generation required (manual MCP tool handlers, WebSocket client, order book logic)
- Manual code appropriate: custom metrics calculations, progressive disclosure logic
- No OpenAPI/SBE schemas to generate from

### III. Modular Architecture ✅ PASS

- Feature-gated: `orderbook` feature flag for this module
- Independent compilation possible
- No cross-module dependencies beyond existing `binance` client

### IV. Type Safety & Contract Enforcement ✅ PASS

- rust_decimal for price/quantity precision (FR-017)
- BTreeMap<Decimal, Decimal> for sorted order book (FR-015)
- Typed entities: OrderBook, OrderBookMetrics, Wall, SlippageEstimate, OrderBookDepth, OrderBookHealth
- JSON Schema via schemars for all MCP tool inputs/outputs (FR-001, FR-002, FR-003)
- Timestamps as i64 milliseconds (FR-018)

### V. MCP Protocol Compliance ✅ PASS

- Tools registered via rmcp ToolRouter (existing pattern)
- JSON Schema for all tool inputs (symbol: String, levels: Option<u32>)
- JSON Schema for all tool outputs (metrics, depth, health)
- No dynamic tool updates (static tool list)
- Stdio transport (default)
- Progress notifications not required (<10s operations)

### VI. Async-First Design ✅ PASS

- Tokio async throughout
- WebSocket streams via tokio-tungstenite
- Async HTTP via reqwest (existing)
- Rate limiter: async-aware (governor 0.10.1, GCRA algorithm)
- Error handling via thiserror (existing pattern)

### VII. Machine-Optimized Development ✅ PASS

- Feature added via `/speckit.specify` (completed)
- Specification has Given/When/Then scenarios (3 user stories with acceptance criteria)
- Numbered requirements: FR-001 through FR-027
- Measurable success criteria: SC-001 through SC-017
- Clarifications resolved (3 questions answered in spec)

**Constitution Compliance**: ✅ **ALL GATES PASSED** - Ready for Phase 0 research

## Project Structure

### Documentation (this feature)

```
specs/007-orderbook-depth-tools/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── get_orderbook_metrics.json
│   ├── get_orderbook_depth.json
│   └── get_orderbook_health.json
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── orderbook/                    # NEW: Order book module
│   ├── mod.rs                   # Module exports
│   ├── types.rs                 # OrderBook, OrderBookMetrics, Wall, SlippageEstimate, OrderBookDepth, OrderBookHealth
│   ├── manager.rs               # OrderBookManager (tracks up to 20 symbols, lazy init)
│   ├── metrics.rs               # Metrics calculations (spread_bps, microprice, imbalance_ratio, walls, slippage)
│   ├── websocket.rs             # WebSocket client for depth streams (<symbol>@depth@100ms)
│   ├── rate_limiter.rs          # Client-side rate limiter (1000 req/min, 30s queue)
│   └── tools.rs                 # MCP tool handlers (get_orderbook_metrics, get_orderbook_depth, get_orderbook_health)
├── binance/                     # EXISTING: Binance API client
│   ├── mod.rs
│   └── client.rs                # BinanceClient (REST API calls)
├── server/                      # EXISTING: MCP server
│   ├── mod.rs
│   ├── handler.rs               # ServerHandler trait impl
│   └── tool_router.rs           # Tool routing (ADD: orderbook tools)
└── main.rs                      # EXISTING: Entry point

tests/
├── integration/
│   ├── orderbook_websocket.rs   # NEW: WebSocket reconnection tests
│   ├── orderbook_rate_limit.rs  # NEW: Rate limiter tests
│   └── orderbook_metrics.rs     # NEW: Metrics calculation accuracy tests
└── unit/
    ├── orderbook_types.rs       # NEW: Entity serialization tests
    └── orderbook_manager.rs     # NEW: Symbol limit, lazy init tests
```

**Structure Decision**: Single project structure (src/ + tests/). This is an MCP server extension, not a separate web/mobile application. The `orderbook/` module integrates with existing `binance/` client and `server/` infrastructure. Feature-gated via `orderbook` feature flag in Cargo.toml (default-enabled for now, can be opt-in later).

## Complexity Tracking

*No violations - Constitution Check passed all gates*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
