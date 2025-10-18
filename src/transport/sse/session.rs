//! SSE connection session manager
//!
//! Manages lifecycle of active SSE connections including:
//! - Connection registration and cleanup
//! - Connection limit enforcement (max 50)
//! - Timeout detection and stale session removal

use super::types::{ConnectionId, SessionMetadata};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Maximum concurrent SSE connections allowed
///
/// Per spec SC-004: "Server handles at least 50 concurrent SSE connections"
pub const MAX_CONNECTIONS: usize = 50;

/// Session timeout in seconds (30s of inactivity)
pub const SESSION_TIMEOUT_SECS: u64 = 30;

/// SSE connection session manager
///
/// Thread-safe manager for tracking active SSE connections.
/// Enforces connection limits and handles session lifecycle.
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<ConnectionId, SessionMetadata>>>,
}

impl SessionManager {
    /// Creates a new empty session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new SSE connection session
    ///
    /// Returns `Some(connection_id)` if registration succeeds,
    /// `None` if max connections limit reached.
    pub async fn register_connection(
        &self,
        client_addr: SocketAddr,
        user_agent: Option<String>,
    ) -> Option<ConnectionId> {
        let mut sessions = self.sessions.write().await;

        // Check connection limit (SC-004)
        if sessions.len() >= MAX_CONNECTIONS {
            tracing::warn!(
                current_connections = sessions.len(),
                max_connections = MAX_CONNECTIONS,
                "Max concurrent connections reached, rejecting new connection"
            );
            return None;
        }

        // Generate unique connection ID
        let connection_id = Uuid::new_v4().to_string();
        let metadata = SessionMetadata::new(connection_id.clone(), client_addr, user_agent);

        sessions.insert(connection_id.clone(), metadata);

        tracing::info!(
            connection_id = %connection_id,
            client_addr = %client_addr,
            total_connections = sessions.len(),
            "SSE connection registered"
        );

        Some(connection_id)
    }

    /// Removes a connection session by ID
    ///
    /// Returns `true` if session existed and was removed, `false` otherwise.
    pub async fn remove_connection(&self, connection_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(connection_id).is_some();

        if removed {
            tracing::info!(
                connection_id = %connection_id,
                remaining_connections = sessions.len(),
                "SSE connection removed"
            );
        }

        removed
    }

    /// Updates last activity timestamp for a connection
    ///
    /// Returns `true` if connection exists, `false` if not found.
    pub async fn update_activity(&self, connection_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(connection_id) {
            session.update_activity();
            tracing::debug!(
                connection_id = %connection_id,
                "Activity timestamp updated"
            );
            true
        } else {
            tracing::warn!(
                connection_id = %connection_id,
                "Cannot update activity: connection not found"
            );
            false
        }
    }

    /// Gets session metadata by connection ID
    ///
    /// Returns `Some(SessionMetadata)` if connection exists, `None` otherwise.
    pub async fn get_session(&self, connection_id: &str) -> Option<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions.get(connection_id).cloned()
    }

    /// Checks if a connection ID is valid (exists and not stale)
    pub async fn is_valid_connection(&self, connection_id: &str) -> bool {
        let sessions = self.sessions.read().await;

        sessions
            .get(connection_id)
            .map(|session| !session.is_stale(SESSION_TIMEOUT_SECS))
            .unwrap_or(false)
    }

    /// Removes all stale connections (inactive >30s)
    ///
    /// Returns number of sessions cleaned up.
    pub async fn cleanup_stale_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();

        sessions.retain(|connection_id, session| {
            let is_stale = session.is_stale(SESSION_TIMEOUT_SECS);
            if is_stale {
                tracing::info!(
                    connection_id = %connection_id,
                    "Removing stale session (inactive >{}s)",
                    SESSION_TIMEOUT_SECS
                );
            }
            !is_stale
        });

        let cleaned = initial_count - sessions.len();
        if cleaned > 0 {
            tracing::info!(
                cleaned_sessions = cleaned,
                remaining_sessions = sessions.len(),
                "Stale session cleanup complete"
            );
        }

        cleaned
    }

    /// Returns current number of active connections
    pub async fn connection_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Gets all active connection IDs
    #[cfg(test)]
    pub async fn get_connection_ids(&self) -> Vec<ConnectionId> {
        self.sessions.read().await.keys().cloned().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_register_connection() {
        let manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let conn_id = manager.register_connection(addr, Some("test-agent".to_string())).await;
        assert!(conn_id.is_some());
        assert_eq!(manager.connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_max_connections_limit() {
        let manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        // Register MAX_CONNECTIONS connections
        for _ in 0..MAX_CONNECTIONS {
            assert!(manager.register_connection(addr, None).await.is_some());
        }

        // 51st connection should be rejected
        assert!(manager.register_connection(addr, None).await.is_none());
    }

    #[tokio::test]
    async fn test_remove_connection() {
        let manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let conn_id = manager.register_connection(addr, None).await.unwrap();
        assert_eq!(manager.connection_count().await, 1);

        let removed = manager.remove_connection(&conn_id).await;
        assert!(removed);
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_update_activity() {
        let manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let conn_id = manager.register_connection(addr, None).await.unwrap();
        sleep(Duration::from_millis(100)).await;

        let updated = manager.update_activity(&conn_id).await;
        assert!(updated);
    }

    #[tokio::test]
    async fn test_is_valid_connection() {
        let manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let conn_id = manager.register_connection(addr, None).await.unwrap();
        assert!(manager.is_valid_connection(&conn_id).await);
        assert!(!manager.is_valid_connection("invalid-id").await);
    }

    #[tokio::test]
    async fn test_cleanup_stale_sessions() {
        let manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        // Register 3 connections
        manager.register_connection(addr, None).await;
        manager.register_connection(addr, None).await;
        manager.register_connection(addr, None).await;

        assert_eq!(manager.connection_count().await, 3);

        // Cleanup should remove 0 (all fresh)
        let cleaned = manager.cleanup_stale_sessions().await;
        assert_eq!(cleaned, 0);
        assert_eq!(manager.connection_count().await, 3);
    }
}
