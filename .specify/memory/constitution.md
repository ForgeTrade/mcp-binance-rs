<!--
Sync Impact Report:
Version: 1.0.0 → 1.1.0
Modified Principles: Added new section "Dependency Management"
Added Sections:
  - Dependency Management > Dependency Versioning Policy
Changes:
  - Added mandatory policy for keeping dependencies up-to-date
  - Documented version update process
  - Listed current dependency standards as of 2025-10-16
Templates Requiring Updates:
  ✅ spec-template.md - No changes required
  ✅ plan-template.md - No changes required (dependency info already tracked)
  ✅ tasks-template.md - No changes required
Follow-up TODOs: None
-->

# Binance MCP Server Constitution

## Core Principles

### I. Security-First (NON-NEGOTIABLE)

**Rules**:
- NEVER store API keys in code or version control; environment variables or secure vaults only
- ALL authenticated Binance requests MUST use HMAC SHA256 signing with timestamp validation
- Rate limiting MUST respect Binance weight system; implement exponential backoff for 429/418 responses
- Input validation MUST occur before any external API call; sanitize all user inputs
- Error messages MUST NOT expose sensitive data (API keys, internal state, stack traces)
- Dependency security audits MUST pass before any release (`cargo audit`)

**Rationale**: Financial APIs handle real money and credentials. A single security breach can cause direct financial loss. Defense-in-depth is mandatory.

### II. Auto-Generation Priority

**Rules**:
- Code generation from authoritative sources (OpenAPI specs, SBE schemas, MCP protocol definitions) is PREFERRED over manual implementation
- Auto-generated code MUST NOT be manually edited; regenerate from source instead
- Manual code limited to: glue logic, MCP server handlers, authentication wrappers, custom error mapping
- All code generation processes MUST be documented in `docs/codegen.md` with reproduction steps
- Generated code directories MUST be clearly marked (e.g., `src/generated/` or `// AUTO-GENERATED: DO NOT EDIT`)

**Rationale**: Manual errors are costly in financial systems. Auto-generation ensures consistency with upstream APIs, reduces maintenance burden, and enables rapid updates when Binance API changes.

### III. Modular Architecture

**Rules**:
- Feature-gated Cargo modules for each Binance service: `spot`, `margin`, `futures-usds`, `futures-coin`, `wallet`, etc.
- Default features: `spot` + `server` + `transport-stdio`
- Each module MUST be independently compilable and testable
- Cross-module dependencies MUST be explicit via feature requirements in Cargo.toml
- MCP server MUST support dynamic tool registration based on enabled features

**Rationale**: Users may only need spot trading or only futures. Forcing monolithic builds wastes compile time and increases binary size. Modularity enables à la carte functionality.

### IV. Type Safety & Contract Enforcement

**Rules**:
- Rust type system MUST enforce Binance API contracts (enums for OrderStatus, OrderType, TimeInForce; newtypes for timestamps, weights)
- JSON Schema generation for all MCP tool inputs/outputs via `schemars` crate
- Deserialization failures MUST be treated as errors, never silently ignored
- All timestamps MUST use typed wrappers distinguishing milliseconds vs microseconds
- Filter validation (LotSize, MinNotional, etc.) MUST occur at compile-time where possible, runtime otherwise

**Rationale**: Type errors in trading systems cause incorrect orders. Rust's type system prevents entire classes of bugs that would be runtime failures in dynamic languages.

### V. MCP Protocol Compliance (NON-NEGOTIABLE)

**Rules**:
- Server MUST implement full MCP lifecycle: `initialize` → capability negotiation → `initialized` notification
- Tool discovery via `tools/list` MUST return accurate JSON Schema for all available tools
- Tool execution via `tools/call` MUST return structured responses with proper error codes
- Dynamic tool updates MUST send `tools/list_changed` notifications when features enabled/disabled at runtime
- MUST support both stdio (default) and streamable HTTP transports
- Progress notifications MUST be used for long-running operations (>2 seconds)

**Rationale**: MCP compliance is mandatory for AI clients (Claude, etc.) to interact with the server. Non-compliance means the server is unusable.

### VI. Async-First Design

**Rules**:
- Tokio async runtime throughout; blocking operations prohibited in async contexts
- All Binance API calls MUST be async with timeout handling
- WebSocket streams MUST use async channels (tokio::sync::mpsc or similar)
- Concurrent requests MUST respect Binance rate limits via semaphore/rate-limiter
- Error handling via `async-trait` and `thiserror` for ergonomic Result propagation

**Rationale**: Financial data is real-time. Blocking I/O causes latency spikes and poor resource utilization. Async enables high concurrency without thread overhead.

### VII. Machine-Optimized Development

**Rules**:
- ALL features MUST be added via `/speckit.specify` workflow; ad-hoc changes prohibited
- Specifications MUST be machine-readable: Given/When/Then scenarios, numbered requirements (FR-###), measurable success criteria (SC-###)
- Code changes MUST map to specific task IDs in `tasks.md`
- Tests (when required) MUST be written BEFORE implementation; Red-Green-Refactor cycle enforced
- Ambiguities flagged as `[NEEDS CLARIFICATION: ...]` in specs; implementation blocks until resolved

**Rationale**: This constitution prioritizes machine efficiency. Clear, structured specifications enable LLMs to generate correct code without guesswork. Humans introduce ambiguity; machines require precision.

## Security Requirements

### Authentication & Authorization

- API keys MUST support all Binance auth types: HMAC, RSA, Ed25519
- Key rotation MUST be supported without server restart
- MCP remote servers (HTTP transport) MUST use OAuth2; stdio transport assumes local trust
- IP whitelisting enforcement via environment configuration

### Data Protection

- Sensitive data (keys, signatures, balances) MUST NOT appear in logs at INFO level; DEBUG only with explicit opt-in
- TLS 1.2+ MUST be enforced for all Binance HTTPS connections
- WebSocket connections MUST validate server certificates

### Operational Security

- Rate limit violations MUST trigger alerts (log at WARN level minimum)
- Unexpected HTTP 403 (WAF) or 418 (IP ban) MUST halt operations and alert operator
- STP (Self-Trade Prevention) modes MUST be configurable per order to prevent wash trading

## Dependency Management

### Dependency Versioning Policy

**Rules**:
- ALL dependencies MUST be kept up-to-date with latest stable versions from crates.io
- Version updates MUST be verified via context7 or crates.io API before applying
- Rust edition MUST track the latest stable edition (currently 2024)
- Major version updates MUST be tested locally before committing
- Security patches (PATCH versions) MUST be applied within 7 days of release
- Breaking changes (MAJOR versions) require review of CHANGELOG and migration guides
- `Cargo.lock` MUST be committed to ensure reproducible builds

**Version Update Process**:
1. Check latest versions via context7 or `https://crates.io/api/v1/crates/[crate_name]`
2. Update version numbers in `Cargo.toml` with specific versions (no wildcards)
3. Run `cargo build` and `cargo test` to verify compatibility
4. Review dependency changelogs for breaking changes
5. Update project documentation if API changes affect usage

**Current Dependency Standards** (as of 2025-10-16):
- rmcp: 0.8.1 (MCP SDK)
- tokio: 1.48.0 (async runtime)
- reqwest: 0.12.24 (HTTP client)
- serde: 1.0.228, serde_json: 1.0.145 (serialization)
- schemars: 1.0.4 (JSON schema)
- thiserror: 2.0.17 (error handling)
- tracing: 0.1.41, tracing-subscriber: 0.3.20 (logging)

**Rationale**: Outdated dependencies introduce security vulnerabilities, miss performance improvements, and accumulate technical debt. Regular updates ensure access to bug fixes, security patches, and ecosystem improvements. Specific versions (not ranges) guarantee reproducible builds across environments.

## Development Workflow

### Specification-Driven Development

1. **Feature Request** → `/speckit.specify` → `spec.md` with user stories, requirements, success criteria
2. **Planning** → `/speckit.plan` → `plan.md` with architecture, dependencies, constitution checks
3. **Task Generation** → `/speckit.tasks` → `tasks.md` with dependency-ordered implementation steps
4. **Implementation** → `/speckit.implement` → Execute tasks, tests first if required
5. **Validation** → Run quickstart.md scenarios, verify all acceptance criteria pass

### Constitution Enforcement

- **Pre-Commit**: `cargo clippy`, `cargo fmt --check`, `cargo audit` MUST pass
- **Pre-PR**: All tasks in `tasks.md` MUST be completed; tests (if required) MUST be green
- **Code Review**: Reviewer MUST verify compliance with all 7 core principles
- **Complexity Justification**: Any violation of core principles MUST be documented in `plan.md` Complexity Tracking table with rationale

### Auto-Generation Workflow

1. Update authoritative source (OpenAPI spec, SBE schema, MCP protocol definition)
2. Run code generator (document commands in `docs/codegen.md`)
3. Verify generated code compiles and passes tests
4. Commit generated code with `chore: regenerate [module] from [source]` message
5. NEVER manually edit generated code; fix source and regenerate instead

## Governance

### Amendment Process

1. Propose changes via discussion (document rationale)
2. Update `.specify/memory/constitution.md` via `/speckit.constitution` command
3. Increment version (MAJOR for principle removal/redefinition, MINOR for new principles, PATCH for clarifications)
4. Update dependent templates (`spec-template.md`, `plan-template.md`, `tasks-template.md`) to reflect changes
5. Migrate existing features to comply with new principles (document migration path)

### Versioning Policy

- **MAJOR (x.0.0)**: Backward-incompatible governance changes (e.g., removing auto-generation principle)
- **MINOR (1.x.0)**: New principles or sections added (e.g., adding observability requirements)
- **PATCH (1.0.x)**: Clarifications, typo fixes, non-semantic refinements

### Compliance Review

- Every feature spec (`spec.md`) MUST include constitution compliance checklist
- Every implementation plan (`plan.md`) MUST document any principle violations and justifications
- Constitution supersedes all other project documentation

### SpecKit Integration

- Use `/speckit.specify` for ALL feature additions; direct code changes prohibited
- Use `/speckit.constitution` for constitutional amendments only
- All specifications follow templates in `.specify/templates/`
- Machine-readable formats (Given/When/Then, FR-###, SC-###) are mandatory

**Version**: 1.1.0 | **Ratified**: 2025-10-16 | **Last Amended**: 2025-10-16
