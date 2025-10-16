# Research: MCP Server Foundation

## rmcp Crate Integration

### Decision: Use rmcp v0.8.0 with macro-based tool registration
### Rationale:
- Official Rust SDK from Model Context Protocol organization
- Production-ready with comprehensive test coverage (see coverage badge in README)
- Declarative macro system reduces boilerplate and ensures type safety
- Built-in JSON Schema generation via schemars integration
- Strong async support with tokio runtime

### Implementation Notes:

**Key Patterns Discovered:**

1. **ServerHandler Trait Implementation**
   - Core trait located at `/rust-sdk/crates/rmcp/src/handler/server.rs`
   - Provides default implementations for most methods
   - Only `get_info()` required for minimal server
   - Tool methods override `call_tool()` and `list_tools()`

2. **Tool Registration via Macros**
   ```rust
   use rmcp::{tool, tool_handler, tool_router};

   #[tool_router]
   impl MyServer {
       #[tool(description = "Tool description here")]
       async fn my_tool(&self, Parameters(args): Parameters<MyArgs>)
           -> Result<CallToolResult, McpError>
       {
           // Implementation
           Ok(CallToolResult::success(vec![Content::text("result")]))
       }
   }

   #[tool_handler]
   impl ServerHandler for MyServer {
       fn get_info(&self) -> ServerInfo {
           ServerInfo {
               protocol_version: ProtocolVersion::V_2024_11_05,
               capabilities: ServerCapabilities::builder()
                   .enable_tools()
                   .build(),
               server_info: Implementation::from_build_env(),
               instructions: Some("Server instructions".to_string()),
           }
       }
   }
   ```

3. **Stdio Transport Setup**
   ```rust
   use rmcp::{ServiceExt, transport::stdio};
   use tokio::io::{stdin, stdout};

   #[tokio::main]
   async fn main() -> Result<()> {
       let service = MyServer::new();
       let server = service.serve(stdio()).await?;
       server.waiting().await?;
       Ok(())
   }
   ```

4. **Error Handling Patterns**
   - Uses `ErrorData as McpError` for MCP protocol errors
   - Supports custom error data with `serde_json::Value`
   - Pattern: `McpError::invalid_params("message", Some(json!({"key": "value"})))`
   - Built-in error types: `method_not_found`, `invalid_params`, `resource_not_found`

5. **JSON Schema Integration**
   - Automatic schema generation from Rust types using schemars
   - Derive `JsonSchema` on parameter structs
   - Example from `/rust-sdk/examples/servers/src/common/counter.rs`:
     ```rust
     #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
     pub struct ToolArgs {
         /// Field documentation becomes schema description
         pub field: String,
     }
     ```

6. **Lifecycle Management**
   - `service.serve(transport).await?` - Initializes and starts server
   - `server.waiting().await?` - Blocks until shutdown
   - `server.cancel().await?` - Graceful shutdown option
   - Supports graceful shutdown via cancellation tokens

### Example Code from rust-sdk:

**Minimal Server** (`/rust-sdk/examples/servers/src/memory_stdio.rs`):
```rust
use std::error::Error;
use rmcp::serve_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let service = MyService::new();
    let io = (tokio::io::stdin(), tokio::io::stdout());
    serve_server(service, io).await?;
    Ok(())
}
```

**Counter Server** (`/rust-sdk/examples/servers/src/counter_stdio.rs`):
```rust
use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let service = Counter::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("serving error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
```

### Alternatives Considered:

1. **Manual Protocol Implementation**
   - Pros: Full control, no dependencies
   - Cons: Significant development time, error-prone, no schema validation
   - Rejected: rmcp provides production-ready implementation

2. **Other Language SDKs (Python, TypeScript)**
   - Pros: Mature ecosystems, potentially simpler
   - Cons: Doesn't align with Rust project goals, runtime overhead
   - Rejected: Project requirement is Rust-based

3. **Lower-level rmcp without macros**
   - Pros: More explicit control
   - Cons: More boilerplate, manual schema generation
   - Rejected: Macros reduce errors and improve maintainability

## Binance API Client

### Decision: Use reqwest for HTTP client with manual endpoint construction
### Rationale:
- Official binance-sdk is complex and auto-generated for full API coverage
- We only need `/api/v3/time` endpoint for foundation
- reqwest is industry-standard, well-maintained, with excellent async support
- Direct HTTP calls give us full control over timeouts and retries
- Lighter dependency footprint than full binance-sdk

### API Endpoint: /api/v3/time

**Base URL:** `https://api.binance.com`

**Full Endpoint:** `GET https://api.binance.com/api/v3/time`

**Weight:** 1 (minimal rate limit impact)

**Parameters:** NONE

**Data Source:** Memory (fast, no database latency)

**Request Format:**
```rust
// No query parameters or body required
GET https://api.binance.com/api/v3/time
```

**Response Format:**
```json
{
  "serverTime": 1499827319559
}
```

**Response Type:**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TimeResponse {
    #[serde(rename = "serverTime")]
    pub server_time: i64,  // Unix timestamp in milliseconds
}
```

**Error Codes (from `/binance-spot-api-docs/rest-api.md`):**
- HTTP 429: Rate limit exceeded
- HTTP 418: IP banned for repeated rate limit violations
- HTTP 5XX: Internal server errors (treat as unknown, may have succeeded)
- -1007: Timeout (request took longer than 10 seconds)

**Rate Limiting:**
- Response headers contain `X-MBX-USED-WEIGHT-(intervalNum)(intervalLetter)`
- Weight of 1 is negligible for testing purposes
- 429 responses include `Retry-After` header with seconds to wait
- Critical: Backoff on 429 to avoid IP ban (HTTP 418)

**Implementation Pattern:**
```rust
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct TimeResponse {
    #[serde(rename = "serverTime")]
    server_time: i64,
}

const BINANCE_BASE_URL: &str = "https://api.binance.com";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

async fn get_server_time(client: &Client) -> Result<i64, Box<dyn std::error::Error>> {
    let response = client
        .get(format!("{}/api/v3/time", BINANCE_BASE_URL))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .json::<TimeResponse>()
        .await?;

    Ok(response.server_time)
}
```

**Example from binance-connector-rust** (`/binance-connector-rust/examples/spot/rest_api/general_api/time.rs`):
```rust
use anyhow::{Context, Result};
use std::env;
use binance_sdk::config::ConfigurationRestApi;
use binance_sdk::spot::SpotRestApi;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("API_KEY").context("API_KEY must be set")?;
    let api_secret = env::var("API_SECRET").context("API_SECRET must be set")?;

    let rest_conf = ConfigurationRestApi::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;

    let rest_client = SpotRestApi::production(rest_conf);
    let response = rest_client.time().await.context("time request failed")?;

    info!(?response.rate_limits, "time rate limits");
    let data = response.data().await?;
    info!(?data, "time data");

    Ok(())
}
```

### Alternatives Considered:

1. **Full binance-sdk crate**
   - Pros: Official, comprehensive API coverage
   - Cons: Heavy (24.0.0 version with extensive features), auto-generated code, overkill for single endpoint
   - Decision: Too complex for initial foundation, consider for future expansion
   - Location: `/binance-connector-rust/Cargo.toml` shows extensive dependencies

2. **hyper (lower-level)**
   - Pros: More control, fewer dependencies
   - Cons: More boilerplate, less ergonomic API
   - Decision: Rejected in favor of reqwest's better ergonomics

3. **ureq (blocking)**
   - Pros: Simpler, synchronous API
   - Cons: Doesn't integrate well with tokio async runtime
   - Decision: Rejected due to async requirement from rmcp

## Async Runtime Setup

### Decision: Use tokio with default multi-threaded runtime
### Rationale:
- rmcp SDK requires tokio runtime (dependency: `tokio = { version = "1", features = ["sync", "macros", "rt", "time"] }`)
- Multi-threaded runtime handles concurrent MCP requests efficiently
- Industry standard for Rust async applications
- Excellent ecosystem support (reqwest, tracing, etc.)

### Implementation Notes:

**Main Function Setup:**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Create and start server
    let service = BinanceServer::new();
    let server = service.serve(stdio()).await?;

    // Wait for shutdown
    server.waiting().await?;

    Ok(())
}
```

**Timeout Handling for External Calls:**
```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout<T, F>(
    future: F,
    duration: Duration,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    match timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err("Request timeout".into()),
    }
}

// Usage in tool
#[tool(description = "Get Binance server time")]
async fn get_time(&self) -> Result<CallToolResult, McpError> {
    let duration = Duration::from_secs(10);

    match timeout(duration, self.client.get_server_time()).await {
        Ok(Ok(time)) => Ok(CallToolResult::success(vec![
            Content::text(format!("Server time: {}", time))
        ])),
        Ok(Err(e)) => Err(McpError::internal_error(
            &format!("API error: {}", e),
            None
        )),
        Err(_) => Err(McpError::internal_error(
            "Request timeout after 10s",
            None
        )),
    }
}
```

**Error Propagation with thiserror:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinanceError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimited(u64),

    #[error("API error: {0}")]
    ApiError(String),
}

// Convert to MCP error
impl From<BinanceError> for McpError {
    fn from(err: BinanceError) -> Self {
        McpError::internal_error(&err.to_string(), None)
    }
}
```

**Graceful Shutdown Pattern:**
```rust
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// In main:
tokio::select! {
    result = server.waiting() => {
        result?;
    }
    _ = shutdown_signal() => {
        tracing::info!("Shutdown signal received");
        server.cancel().await?;
    }
}
```

### Alternatives Considered:

1. **Single-threaded runtime** (`#[tokio::main(flavor = "current_thread")]`)
   - Pros: Lower overhead, simpler
   - Cons: Cannot handle concurrent requests efficiently
   - Decision: Rejected; MCP servers should handle concurrent tool calls

2. **async-std**
   - Pros: Alternative async runtime
   - Cons: rmcp requires tokio, ecosystem smaller
   - Decision: Not compatible with rmcp requirements

3. **Custom runtime configuration**
   - Pros: Fine-tuned performance
   - Cons: Added complexity, default is well-optimized
   - Decision: Use defaults unless profiling shows issues

## Credential Management

### Decision: Use std::env with clear error messages, no .env file loading
### Rationale:
- MCP servers run in controlled environments (Claude Desktop config)
- Environment variables set by parent process (no need for .env parsing)
- Keep dependencies minimal
- Clear error messages guide users to proper configuration
- Prevents accidental credential exposure in logs

### Implementation Notes:

**Environment Variable Loading:**
```rust
use std::env;

pub struct BinanceConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
}

impl BinanceConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("BINANCE_API_KEY")
            .map_err(|_| "BINANCE_API_KEY environment variable not set")?;

        let secret_key = env::var("BINANCE_SECRET_KEY")
            .map_err(|_| "BINANCE_SECRET_KEY environment variable not set")?;

        let base_url = env::var("BINANCE_BASE_URL")
            .unwrap_or_else(|_| "https://api.binance.com".to_string());

        // Validate not empty
        if api_key.is_empty() {
            return Err("BINANCE_API_KEY is empty".into());
        }
        if secret_key.is_empty() {
            return Err("BINANCE_SECRET_KEY is empty".into());
        }

        Ok(Self {
            api_key,
            secret_key,
            base_url,
        })
    }
}
```

**Logging Safety Patterns:**
```rust
use tracing::{info, error, debug};

// NEVER log credentials directly
// BAD: info!("API Key: {}", api_key);

// GOOD: Mask credentials in logs
fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &key[..4], &key[key.len()-4..])
}

info!("Loaded API key: {}", mask_key(&config.api_key));

// GOOD: Use structured logging without values
info!("Configuration loaded successfully");

// GOOD: Debug-only logging with conditional compilation
#[cfg(debug_assertions)]
debug!("Config: api_key present={}, base_url={}",
    !config.api_key.is_empty(),
    config.base_url
);
```

**Claude Desktop Configuration Example:**
```json
{
  "mcpServers": {
    "binance": {
      "command": "/path/to/mcp-binance-server",
      "args": [],
      "env": {
        "BINANCE_API_KEY": "your_api_key_here",
        "BINANCE_SECRET_KEY": "your_secret_key_here",
        "BINANCE_BASE_URL": "https://api.binance.com",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Error Messages for Missing Variables:**
```rust
impl BinanceConfig {
    pub fn from_env() -> Result<Self, String> {
        let api_key = env::var("BINANCE_API_KEY").map_err(|_| {
            "BINANCE_API_KEY not set. Configure in Claude Desktop MCP settings:\n\
             \"env\": { \"BINANCE_API_KEY\": \"your_key\" }"
        })?;

        let secret_key = env::var("BINANCE_SECRET_KEY").map_err(|_| {
            "BINANCE_SECRET_KEY not set. Configure in Claude Desktop MCP settings:\n\
             \"env\": { \"BINANCE_SECRET_KEY\": \"your_secret\" }"
        })?;

        // ... similar for other vars
    }
}
```

### Alternatives Considered:

1. **dotenvy crate for .env file loading**
   - Pros: Developer-friendly, common pattern
   - Cons: Not needed in MCP environment, adds dependency, risk of committing .env files
   - Decision: Rejected; MCP servers receive env vars from parent process
   - Example usage would be: `dotenvy::dotenv().ok();` in main

2. **config crate with TOML/YAML**
   - Pros: More structured configuration
   - Cons: Overkill for simple credential management, another file to manage
   - Decision: Rejected; env vars sufficient for MCP use case

3. **Hardcoded defaults**
   - Pros: Simplest implementation
   - Cons: Security risk, inflexible
   - Decision: Rejected; violates security best practices

## Dependencies Final List

```toml
[package]
name = "mcp-binance-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# MCP Server SDK - Official Rust implementation
# Provides ServerHandler trait, tool macros, stdio transport
rmcp = { version = "0.8", features = ["server", "macros"] }

# Async runtime - Required by rmcp, industry standard
tokio = { version = "1", features = ["full"] }

# HTTP client - For Binance API calls
# Features: json (auto serialize/deserialize), rustls-tls (TLS support)
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Serialization - Required for JSON handling
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# JSON Schema generation - Auto-generates schemas for tool parameters
# Used by rmcp macros for MCP protocol compliance
schemars = { version = "1.0", features = ["chrono04"] }

# Error handling - Ergonomic error types with context
thiserror = "2"

# Logging - Structured logging for debugging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[profile.release]
# Optimize for binary size and performance
lto = true
codegen-units = 1
strip = true
```

**Dependency Justifications:**

1. **rmcp 0.8** (Required)
   - Official MCP SDK, core functionality
   - Provides ServerHandler trait, macros, transport layers
   - Version 0.8 is current stable (see `/rust-sdk/README.md`)

2. **tokio 1.x** (Required by rmcp)
   - Async runtime required by rmcp
   - "full" feature includes all needed capabilities
   - Industry standard, well-maintained

3. **reqwest 0.12** (HTTP Client)
   - Most popular Rust HTTP client
   - Excellent async/await support
   - Built-in JSON serialization
   - rustls-tls avoids OpenSSL dependency

4. **serde/serde_json 1.0** (Required)
   - JSON serialization/deserialization
   - Required for MCP protocol and Binance API
   - De facto standard in Rust ecosystem

5. **schemars 1.0** (Required by rmcp)
   - JSON Schema generation from Rust types
   - Enables automatic parameter validation
   - Required for MCP tool definitions

6. **thiserror 2** (Error Handling)
   - Ergonomic error type derivation
   - Reduces boilerplate for custom errors
   - Good error messages for users

7. **tracing/tracing-subscriber** (Observability)
   - Structured logging (better than println!)
   - Essential for debugging MCP protocol issues
   - env-filter allows runtime log level control via RUST_LOG

**Total Dependencies:** 7 direct dependencies (lean and focused)

**Build Time:** Expected < 2 minutes on modern hardware (based on rmcp test compilation times)

**Binary Size:** Expected ~5-10 MB release build (with strip=true and LTO)

## Additional Implementation Recommendations

### Project Structure
```
mcp-binance-server/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, runtime setup
│   ├── server.rs         # ServerHandler implementation
│   ├── tools/
│   │   └── time.rs       # get_time tool implementation
│   ├── client.rs         # Binance HTTP client wrapper
│   ├── config.rs         # Environment variable loading
│   └── error.rs          # Error types
├── tests/
│   └── integration.rs    # Integration tests
└── README.md             # Setup instructions
```

### Testing Strategy
1. **Unit Tests:** Test individual functions (config loading, error conversion)
2. **Integration Tests:** Test tool execution with mock HTTP responses
3. **Manual Testing:** Use MCP Inspector for interactive testing
4. **Production Testing:** Test with Claude Desktop

### Documentation Requirements
1. **README.md:** Installation, configuration, usage examples
2. **Code Comments:** Focus on "why" not "what"
3. **Error Messages:** Clear instructions for users
4. **Claude Desktop Setup:** Example configuration JSON

### Security Considerations
1. Never log credentials (use masking functions)
2. Validate environment variables on startup
3. Use HTTPS only (enforce in client)
4. Handle rate limiting gracefully (respect 429 responses)
5. Set reasonable timeouts (10s default)

### Performance Targets
1. Server startup: < 1 second
2. `/api/v3/time` response: < 500ms (network dependent)
3. Memory footprint: < 50 MB
4. Handle concurrent requests (multi-threaded tokio)

### Future Extension Points
1. Additional Binance endpoints (market data, trading)
2. WebSocket support for real-time data
3. Caching layer for frequently accessed data
4. Metrics collection (request counts, latencies)
5. Configuration hot-reload

## References

### Documentation
- MCP Specification: https://spec.modelcontextprotocol.io/
- rmcp crate docs: https://docs.rs/rmcp
- Binance API docs: `/binance-spot-api-docs/rest-api.md`
- Tokio docs: https://tokio.rs

### Example Code Locations
- rmcp examples: `/rust-sdk/examples/servers/`
- Counter server: `/rust-sdk/examples/servers/src/counter_stdio.rs`
- Binance time example: `/binance-connector-rust/examples/spot/rest_api/general_api/time.rs`
- Tool macros: `/rust-sdk/examples/servers/src/common/counter.rs`

### Key Files Reviewed
1. `/rust-sdk/crates/rmcp/Cargo.toml` - Dependency versions
2. `/rust-sdk/crates/rmcp/src/handler/server.rs` - ServerHandler trait
3. `/binance-spot-api-docs/rest-api.md` - API specification
4. `/binance-connector-rust/src/spot/rest_api/models/time_response.rs` - Response types
