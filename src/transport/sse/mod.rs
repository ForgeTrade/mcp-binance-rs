//! Server-Sent Events (SSE) transport implementation for MCP
//!
//! This module provides HTTPS-based remote access to the Binance MCP server
//! using Server-Sent Events (SSE) protocol. It enables deployment to cloud
//! platforms like Shuttle.dev with automatic HTTPS and zero SSL configuration.
//!
//! ## Architecture
//!
//! ```text
//! Client (Claude Desktop)
//!   ↓ HTTPS GET /mcp/sse
//! SSE Server (rmcp SDK)
//!   ↓ Connection established, returns connection-id
//! Client → POST /mcp/message (with connection-id header)
//!   ↓ JSON-RPC 2.0 message
//! BinanceServer handlers
//!   ↓ Tool execution
//! Response via SSE event stream
//! ```
//!
//! ## Modules
//!
//! - `types`: Connection session types and metadata
//! - `server`: SSE server configuration and setup
//! - `session`: Connection lifecycle management
//! - `handlers`: HTTP endpoint handlers (T020-T022)
//! - `stream`: SSE event stream writer (T022)

pub mod types;
pub mod server;
pub mod session;
//pub mod handlers; // Complex version - deferred to polish phase
pub mod handlers_simple; // MVP implementation

// Re-export main types for convenience
pub use server::SseConfig;
pub use session::SessionManager;
pub use types::{ConnectionId, SessionMetadata};
pub use handlers_simple::{SseState, message_post, tools_list, server_info};
