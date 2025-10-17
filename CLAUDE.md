# mcp-binance-rs Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-16

## Active Technologies
- Rust 1.90+ (Edition 2024) - Updated from 1.75+ with 2021 edition
- rmcp 0.8.1 - MCP Server SDK with macros
- tokio 1.48.0 - Async runtime with full features
- axum 0.8+ - HTTP server framework (003-specify-scripts-bash)
- tokio-tungstenite 0.26+ - WebSocket client (003-specify-scripts-bash)
- tower 0.5+ - Middleware stack (003-specify-scripts-bash)
- tower-http 0.6+ - HTTP-specific middleware (003-specify-scripts-bash)
- reqwest 0.12.24 - HTTP client (json + rustls-tls)
- serde 1.0.228 + serde_json 1.0.145 - Serialization
- schemars 1.0.4 - JSON Schema generation
- chrono 0.4 - Date and time library with serde support
- thiserror 2.0.17 - Error handling
- tracing 0.1.41 + tracing-subscriber 0.3.20 - Logging
- N/A (tests use Binance Testnet API and in-memory mock servers) (004-specify-scripts-bash)

## Project Structure
```
src/
tests/
```

## Commands
cargo build --features http,websocket
cargo run --features http,websocket
cargo test
cargo clippy

## Code Style
Rust 1.75+ (Edition 2024): Follow standard conventions

## Dependency Management Policy
- ALL dependencies MUST be kept up-to-date with latest stable versions
- Version updates verified via context7 or crates.io API
- Security patches applied within 7 days
- Major version updates require CHANGELOG review
- See constitution.md § Dependency Management for full policy

## Recent Changes
- 004-specify-scripts-bash: Added Rust 1.90+ (Edition 2024)
- 004-specify-scripts-bash: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]
- 003-specify-scripts-bash: Added HTTP REST API and WebSocket support (axum 0.8+, tokio-tungstenite 0.26+, tower 0.5+, tower-http 0.6+)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
