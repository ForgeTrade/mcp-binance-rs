//! SSE connection session types and metadata
//!
//! This module defines core data structures for managing SSE connections,
//! including connection IDs, session metadata, and connection state.

use std::net::SocketAddr;
use std::time::SystemTime;

/// Unique identifier for an SSE connection session
///
/// Generated using UUID v4 when a client connects to `/mcp/sse`.
/// Used as the `X-Connection-ID` header value for message routing.
pub type ConnectionId = String;

/// Metadata for an active SSE connection session
///
/// Tracks connection lifecycle, client information, and activity timestamps.
/// Sessions are stored in-memory and cleaned up on disconnect or timeout.
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// Unique connection identifier (UUID v4)
    pub connection_id: ConnectionId,

    /// Client's IP address and port
    pub client_addr: SocketAddr,

    /// Timestamp when connection was established
    pub connected_at: SystemTime,

    /// Timestamp of last message exchange
    ///
    /// Updated on every incoming message. Used for timeout detection (>30s = stale).
    pub last_activity: SystemTime,

    /// HTTP User-Agent header value (optional)
    ///
    /// Useful for debugging and logging client types.
    pub user_agent: Option<String>,
}

impl SessionMetadata {
    /// Creates new session metadata with current timestamp
    pub fn new(
        connection_id: ConnectionId,
        client_addr: SocketAddr,
        user_agent: Option<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            connection_id,
            client_addr,
            connected_at: now,
            last_activity: now,
            user_agent,
        }
    }

    /// Updates last_activity timestamp to current time
    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now();
    }

    /// Checks if session has timed out (no activity for >30 seconds)
    pub fn is_stale(&self, timeout_secs: u64) -> bool {
        self.last_activity
            .elapsed()
            .map(|duration| duration.as_secs() > timeout_secs)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_session_metadata_creation() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let session = SessionMetadata::new(
            "test-conn-id".to_string(),
            addr,
            Some("test-agent".to_string()),
        );

        assert_eq!(session.connection_id, "test-conn-id");
        assert_eq!(session.client_addr, addr);
        assert_eq!(session.user_agent, Some("test-agent".to_string()));
        assert!(!session.is_stale(30));
    }

    #[test]
    fn test_activity_update() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut session = SessionMetadata::new("test-conn-id".to_string(), addr, None);

        let initial_time = session.last_activity;
        thread::sleep(Duration::from_millis(100));
        session.update_activity();

        assert!(session.last_activity > initial_time);
    }

    #[test]
    fn test_stale_detection() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let session = SessionMetadata::new("test-conn-id".to_string(), addr, None);

        // Fresh session should not be stale
        assert!(!session.is_stale(30));

        // Immediate timeout should mark as stale
        assert!(session.is_stale(0));
    }
}
