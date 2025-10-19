//! SSE connection session manager
//!
//! Manages lifecycle of active SSE connections including:
//! - Connection registration and cleanup
//! - Connection limit enforcement (max 50)
//! - Timeout detection and stale session removal
//! - Per-session credential storage (Feature 011)

use super::types::{ConnectionId, SessionMetadata};
pub use crate::types::Environment; // Re-export for credential tools
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Session-scoped API credentials for Binance authentication
///
/// Credentials are stored per-session and cleared when session ends (FR-003, FR-004).
/// API secrets are never logged at any log level (NFR-002).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Binance API key (validated: 64 alphanumeric characters)
    pub api_key: String,

    /// Binance API secret (validated: 64 alphanumeric characters, never logged)
    pub api_secret: String,

    /// Target Binance environment (testnet or mainnet)
    pub environment: Environment,

    /// ISO8601 timestamp when credentials were configured
    pub configured_at: DateTime<Utc>,

    /// UUID v4 session ID for isolation (references Mcp-Session-Id header)
    /// Never serialized in responses for security
    #[serde(skip)]
    pub session_id: String,
}

impl Credentials {
    /// Creates new credentials for a session
    ///
    /// # Arguments
    ///
    /// * `api_key` - Binance API key (must be validated before calling)
    /// * `api_secret` - Binance API secret (must be validated before calling)
    /// * `environment` - Target environment (Testnet or Mainnet)
    /// * `session_id` - Session UUID for isolation
    pub fn new(
        api_key: String,
        api_secret: String,
        environment: Environment,
        session_id: String,
    ) -> Self {
        Self {
            api_key,
            api_secret,
            environment,
            configured_at: Utc::now(),
            session_id,
        }
    }

    /// Returns first 8 characters of API key for status display (NFR-003)
    ///
    /// Used by `get_credentials_status` tool to show key prefix without exposing full key.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_binance_server::transport::sse::session::Credentials;
    /// use mcp_binance_server::types::Environment;
    ///
    /// let creds = Credentials::new(
    ///     "ABCDEFGHabcdefgh12345678901234567890123456789012345678901234".to_string(),
    ///     "secret123456789012345678901234567890123456789012345678901234".to_string(),
    ///     Environment::Testnet,
    ///     "session-id".to_string(),
    /// );
    /// assert_eq!(creds.key_prefix(), "ABCDEFGH");
    /// ```
    pub fn key_prefix(&self) -> String {
        self.api_key.chars().take(8).collect()
    }
}

/// SSE connection session manager
///
/// Thread-safe manager for tracking active SSE connections and per-session credentials.
/// Enforces connection limits and handles session lifecycle with secure credential storage.
#[derive(Clone)]
pub struct SessionManager {
    /// Active SSE connection sessions (Feature 010)
    sessions: Arc<RwLock<HashMap<ConnectionId, SessionMetadata>>>,

    /// Per-session API credentials (Feature 011)
    /// - Key: Session ID (Mcp-Session-Id header)
    /// - Value: Credentials (api_key, api_secret, environment)
    /// - Cleared atomically when session expires (FR-003, FR-004)
    credentials: Arc<RwLock<HashMap<ConnectionId, Credentials>>>,
}

impl SessionManager {
    /// Creates a new empty session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
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
    /// Atomically removes both session metadata AND credentials (Feature 011 - T010).
    ///
    /// Returns `true` if session existed and was removed, `false` otherwise.
    pub async fn remove_connection(&self, connection_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(connection_id).is_some();

        if removed {
            // Atomically remove credentials when session is removed (FR-003, FR-004)
            let mut creds = self.credentials.write().await;
            let had_credentials = creds.remove(connection_id).is_some();

            tracing::info!(
                connection_id = %connection_id,
                remaining_connections = sessions.len(),
                credentials_cleared = had_credentials,
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
    /// Atomically removes both session metadata AND credentials (Feature 011 - T010).
    ///
    /// Returns number of sessions cleaned up.
    pub async fn cleanup_stale_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();

        // Collect stale session IDs
        let stale_ids: Vec<String> = sessions
            .iter()
            .filter_map(|(connection_id, session)| {
                if session.is_stale(SESSION_TIMEOUT_SECS) {
                    Some(connection_id.clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove stale sessions
        for connection_id in &stale_ids {
            sessions.remove(connection_id);
            tracing::info!(
                connection_id = %connection_id,
                "Removing stale session (inactive >{}s)",
                SESSION_TIMEOUT_SECS
            );
        }

        // Atomically remove credentials for stale sessions (FR-003, FR-004)
        let mut creds = self.credentials.write().await;
        let mut credentials_cleared = 0;
        for connection_id in &stale_ids {
            if creds.remove(connection_id).is_some() {
                credentials_cleared += 1;
            }
        }

        let cleaned = stale_ids.len();
        if cleaned > 0 {
            tracing::info!(
                cleaned_sessions = cleaned,
                credentials_cleared = credentials_cleared,
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

    /// Stores credentials for a session (Feature 011 - T007)
    ///
    /// If credentials already exist for this session, they are replaced (last write wins).
    /// A warning is logged when overwriting existing credentials.
    ///
    /// # Arguments
    ///
    /// * `credentials` - Validated credentials to store
    ///
    /// # Returns
    ///
    /// `true` if credentials were stored, `false` if session doesn't exist
    pub async fn store_credentials(&self, credentials: Credentials) -> bool {
        let session_id = credentials.session_id.clone();

        // STEP 1: Verify session exists before storing credentials
        // Security: Prevents credential storage for non-existent or expired sessions
        // Locking: Uses short-lived read lock to minimize contention
        {
            let sessions = self.sessions.read().await;
            if !sessions.contains_key(&session_id) {
                tracing::warn!(
                    session_id = %session_id,
                    "Cannot store credentials: session not found"
                );
                return false;
            }
            // Read lock released here - important to not hold multiple locks simultaneously
        }

        // STEP 2: Store credentials with write lock
        // Locking strategy: Separate scope from session check to avoid deadlocks
        // Security: Credentials stored in memory only (NFR-002), never persisted to disk
        let mut creds = self.credentials.write().await;
        let is_replacement = creds.contains_key(&session_id);

        // Last-write-wins behavior: If credentials already exist, replace them
        // This allows users to reconfigure without explicitly revoking first
        if is_replacement {
            tracing::warn!(
                session_id = %session_id,
                environment = %credentials.environment,
                "Replacing existing credentials for session (last write wins)"
            );
        }

        creds.insert(session_id.clone(), credentials);

        tracing::info!(
            session_id = %session_id,
            "Credentials stored for session"
        );

        true
    }

    /// Retrieves credentials for a session (Feature 011 - T008)
    ///
    /// Returns a clone of credentials to avoid holding lock during HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to retrieve credentials for
    ///
    /// # Returns
    ///
    /// `Some(Credentials)` if credentials exist, `None` otherwise
    pub async fn get_credentials(&self, session_id: &str) -> Option<Credentials> {
        // Locking strategy: Short-lived read lock + clone pattern
        // Why clone? Allows HTTP requests to use credentials without holding lock,
        // preventing lock contention during slow network operations (100ms+ latency).
        // Trade-off: Small memory overhead (3 strings ~200 bytes) for better concurrency.
        let creds = self.credentials.read().await;
        creds.get(session_id).cloned()
        // Read lock released immediately after clone - HTTP requests execute lock-free
    }

    /// Revokes credentials from a session (Feature 011 - T009)
    ///
    /// Removes credentials from memory without closing the session.
    /// Session remains active for public API calls.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to revoke credentials from
    ///
    /// # Returns
    ///
    /// `true` if credentials existed and were removed, `false` if no credentials found
    pub async fn revoke_credentials(&self, session_id: &str) -> bool {
        // Locking strategy: Write lock required for HashMap::remove()
        // Security: Immediate removal from memory ensures credentials no longer usable
        // Idempotent: Safe to call multiple times - returns false if already removed
        let mut creds = self.credentials.write().await;
        let removed = creds.remove(session_id).is_some();

        if removed {
            tracing::info!(
                session_id = %session_id,
                "Credentials revoked from session"
            );
        } else {
            // Not an error - idempotent behavior allows safe retry
            tracing::debug!(
                session_id = %session_id,
                "No credentials to revoke for session"
            );
        }

        removed
        // Write lock released - credentials permanently removed from memory
        // Session continues to exist and can be used for public API calls
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

        let conn_id = manager
            .register_connection(addr, Some("test-agent".to_string()))
            .await;
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
