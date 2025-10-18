//! Transport layer implementations for MCP communication
//!
//! This module provides different transport mechanisms for MCP protocol:
//! - stdio: Standard input/output transport (default, local-only)
//! - sse: Server-Sent Events transport for HTTPS remote access (feature-gated)

pub mod stdio;

#[cfg(feature = "sse")]
pub mod sse;
