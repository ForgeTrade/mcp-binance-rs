# mcp-binance-rs Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-16

## Active Technologies
- Rust 1.90+ (Edition 2024) - Updated from 1.75+ with 2021 edition
- rmcp 0.8.1 - MCP Server SDK with macros
- tokio 1.48.0 - Async runtime with full features
- reqwest 0.12.24 - HTTP client (json + rustls-tls)
- serde 1.0.228 + serde_json 1.0.145 - Serialization
- schemars 1.0.4 - JSON Schema generation
- thiserror 2.0.17 - Error handling
- tracing 0.1.41 + tracing-subscriber 0.3.20 - Logging

## Project Structure
```
src/
tests/
```

## Commands
cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style
Rust 1.75+ (Edition 2024): Follow standard conventions

## Dependency Management Policy
- ALL dependencies MUST be kept up-to-date with latest stable versions
- Version updates verified via context7 or crates.io API
- Security patches applied within 7 days
- Major version updates require CHANGELOG review
- See constitution.md § Dependency Management for full policy

## Recent Changes
- 2025-10-16: Updated all dependencies to latest versions (rmcp 0.8.1, tokio 1.48.0, reqwest 0.12.24, serde 1.0.228, serde_json 1.0.145, schemars 1.0.4, thiserror 2.0.17, tracing 0.1.41, tracing-subscriber 0.3.20)
- 2025-10-16: Upgraded Rust edition from 2021 to 2024
- 2025-10-16: Added Dependency Management section to constitution (v1.1.0)
- 001-mcp-server-foundation: Added Rust 1.75+ (Edition 2024)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
