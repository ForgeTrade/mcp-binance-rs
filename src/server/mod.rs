//! MCP Server Implementation
//!
//! This module contains the MCP server infrastructure including the ServerHandler
//! trait implementation and tool routing logic.

pub mod handler;
pub mod resources;
pub mod tool_router;
pub mod types;

use crate::binance::BinanceClient;
use crate::config::Credentials;
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::router::tool::ToolRouter;

#[cfg(feature = "sse")]
use crate::transport::sse::session::SessionManager;

#[cfg(feature = "orderbook")]
use std::sync::Arc;

#[cfg(feature = "orderbook")]
use crate::orderbook::OrderBookManager;

#[cfg(feature = "orderbook_analytics")]
use crate::orderbook::analytics::storage::SnapshotStorage;

/// Main Binance MCP Server struct
///
/// This struct holds the server state including Binance API client, credentials,
/// and routers for handling MCP requests.
#[derive(Clone)]
pub struct BinanceServer {
    /// Binance API client for making requests
    pub binance_client: BinanceClient,
    /// Optional API credentials loaded from environment
    pub credentials: Option<Credentials>,
    /// Session manager for per-session credential storage (Feature 011, SSE only)
    #[cfg(feature = "sse")]
    pub session_manager: SessionManager,
    /// Tool router for MCP tool routing
    pub tool_router: ToolRouter<Self>,
    /// Prompt router for MCP prompt routing
    pub prompt_router: PromptRouter<Self>,
    /// Order book manager for depth analysis (feature-gated)
    #[cfg(feature = "orderbook")]
    pub orderbook_manager: Arc<OrderBookManager>,
    /// Snapshot storage for analytics (feature-gated)
    #[cfg(feature = "orderbook_analytics")]
    pub snapshot_storage: Arc<SnapshotStorage>,
}

impl BinanceServer {
    /// Creates a new Binance server instance
    ///
    /// Loads credentials from environment variables and initializes Binance API client.
    /// Logs credential status at INFO level (masked), or WARN if not configured.
    pub fn new() -> Self {
        let credentials = match Credentials::from_env() {
            Ok(creds) => {
                // T009: Log at INFO level with masked key (NEVER log secret_key)
                tracing::info!(
                    "API credentials configured (key: {})",
                    creds.api_key // Display trait shows masked version
                );
                Some(creds)
            }
            Err(err) => {
                // T010: Handle missing credentials gracefully
                tracing::warn!(
                    "No API credentials configured; authenticated features disabled. {}",
                    err
                );
                None
            }
        };

        let binance_client = BinanceClient::new();

        #[cfg(feature = "orderbook")]
        let orderbook_manager = Arc::new(OrderBookManager::new(Arc::new(binance_client.clone())));

        #[cfg(feature = "orderbook_analytics")]
        let snapshot_storage = {
            let storage_path = std::env::var("ORDERBOOK_STORAGE_PATH")
                .unwrap_or_else(|_| "./data/orderbook_snapshots".to_string());
            Arc::new(
                SnapshotStorage::new(std::path::Path::new(&storage_path))
                    .expect("Failed to initialize snapshot storage"),
            )
        };

        Self {
            binance_client,
            credentials,
            #[cfg(feature = "sse")]
            session_manager: SessionManager::new(),
            tool_router: Self::tool_router(),
            prompt_router: Self::create_prompt_router(),
            #[cfg(feature = "orderbook")]
            orderbook_manager,
            #[cfg(feature = "orderbook_analytics")]
            snapshot_storage,
        }
    }

    /// Checks if the server has valid API credentials configured
    pub fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }
}

impl Default for BinanceServer {
    fn default() -> Self {
        Self::new()
    }
}
