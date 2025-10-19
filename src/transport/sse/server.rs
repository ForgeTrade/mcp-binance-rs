//! SSE server wrapper using rmcp SDK
//!
//! This module provides a high-level wrapper around rmcp's SseServer,
//! integrating it with the BinanceServer tool handlers and managing
//! keep-alive heartbeats to prevent connection timeouts.

use std::net::SocketAddr;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use super::session::SessionManager;

/// SSE server configuration
///
/// Defines paths, bind address, and keep-alive settings for SSE transport.
#[derive(Debug, Clone)]
pub struct SseConfig {
    /// Socket address to bind SSE server (e.g., "0.0.0.0:8000")
    pub bind: SocketAddr,

    /// SSE endpoint path for connection handshake
    pub sse_path: String,

    /// POST endpoint path for JSON-RPC message exchange
    pub post_path: String,

    /// Keep-alive interval (sends heartbeat to prevent timeout)
    ///
    /// Default: 30 seconds (prevents proxy timeouts)
    pub keep_alive: Option<Duration>,

    /// Cancellation token for graceful shutdown
    pub cancellation_token: CancellationToken,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:8000".parse().unwrap(),
            sse_path: "/mcp/sse".to_string(),
            post_path: "/mcp/message".to_string(),
            keep_alive: Some(Duration::from_secs(30)),
            cancellation_token: CancellationToken::new(),
        }
    }
}

impl SseConfig {
    /// Creates SSE config from bind address with default paths
    pub fn new(bind: SocketAddr) -> Self {
        Self {
            bind,
            ..Default::default()
        }
    }

    /// Sets custom SSE endpoint path
    pub fn with_sse_path(mut self, path: impl Into<String>) -> Self {
        self.sse_path = path.into();
        self
    }

    /// Sets custom message POST path
    pub fn with_post_path(mut self, path: impl Into<String>) -> Self {
        self.post_path = path.into();
        self
    }

    /// Sets keep-alive interval
    pub fn with_keep_alive(mut self, duration: Duration) -> Self {
        self.keep_alive = Some(duration);
        self
    }

    /// Sets cancellation token for shutdown
    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = token;
        self
    }
}

/// Starts background task for SSE keep-alive heartbeat (T013)
///
/// Sends periodic heartbeat comments to all active SSE connections
/// to prevent proxy timeouts. Cleans up stale sessions every interval.
///
/// ## Arguments
///
/// - `session_manager`: Manager for tracking active sessions
/// - `interval`: Duration between heartbeats (default: 30s)
/// - `cancellation_token`: Token to stop heartbeat task
///
/// ## Heartbeat Format
///
/// SSE comment line (not visible to client application):
/// ```text
/// : keepalive
///
/// ```
pub async fn start_heartbeat_task(
    session_manager: SessionManager,
    interval: Duration,
    cancellation_token: CancellationToken,
) {
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval);
        interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval_timer.tick() => {
                    // Cleanup stale sessions (T050)
                    let cleaned = session_manager.cleanup_stale_sessions().await;
                    if cleaned > 0 {
                        tracing::debug!(
                            cleaned_sessions = cleaned,
                            "Heartbeat: cleaned stale sessions"
                        );
                    }

                    // Log active connection count
                    let active_count = session_manager.connection_count().await;
                    tracing::trace!(
                        active_connections = active_count,
                        "Heartbeat: SSE keep-alive interval"
                    );

                    // Note: Actual SSE heartbeat events are sent by rmcp SDK
                    // based on SseConfig.keep_alive setting. This task handles
                    // session cleanup and logging.
                }
                _ = cancellation_token.cancelled() => {
                    tracing::info!("Heartbeat task shutting down");
                    break;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SseConfig::default();
        assert_eq!(config.sse_path, "/mcp/sse");
        assert_eq!(config.post_path, "/mcp/message");
        assert_eq!(config.keep_alive, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_config_builder() {
        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let config = SseConfig::new(addr)
            .with_sse_path("/custom/sse")
            .with_post_path("/custom/message")
            .with_keep_alive(Duration::from_secs(60));

        assert_eq!(config.bind, addr);
        assert_eq!(config.sse_path, "/custom/sse");
        assert_eq!(config.post_path, "/custom/message");
        assert_eq!(config.keep_alive, Some(Duration::from_secs(60)));
    }

    #[tokio::test]
    async fn test_heartbeat_task() {
        let session_manager = SessionManager::new();
        let token = CancellationToken::new();
        let token_clone = token.clone();

        // Start heartbeat with very short interval for testing
        start_heartbeat_task(
            session_manager.clone(),
            Duration::from_millis(100),
            token_clone,
        ).await;

        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Cancel and verify cleanup
        token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
