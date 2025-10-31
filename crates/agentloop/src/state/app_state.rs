//! Defines the shared application state.
//!
//! Contains configuration and shared data structures like session states
//! and WebSocket connection maps, protected for concurrent access.
//! Adheres to the one-item-per-file guideline and uses fully qualified paths.

//! Revision History
//! - 2025-05-13T19:19:57Z @AI: Add external and merged tool fields; refactor internal tool loading.
//! - 2025-04-24T14:38:09Z @AI: Add ws_connections field and implement new().

// Required for schema generation and serialization of helper structs
// These were for local structs in `new()`, which are now removed.
// use schemars::JsonSchema;
// use serde::{Serialize, Deserialize};

/// Shared application state accessible across handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Application configuration settings.
    pub config: crate::config::app_config::AppConfig,
    /// Map of active sessions, keyed by SessionId.
    pub sessions: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<crate::types::session_id::SessionId, crate::types::session_data::SessionData>>>,
    /// Map of active WebSocket connections, keyed by SessionId.
    /// Each value is a list of recipients (client connections) for that session.
    pub ws_connections: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<crate::types::session_id::SessionId, std::vec::Vec<actix::Recipient<crate::types::ws_event::WebsocketEvent>>>>>,
    /// All tool definitions (internal + external), sorted by name, for LLM prompting.
    pub tool_schemas: std::sync::Arc<Option<crate::tools::tools_schema::ToolsSchema>>,
    /// Merged handlers for all tools (internal + external), for dispatch.
    pub tool_handler: std::sync::Arc<Option<crate::tools::tool_handler::ToolHandler>>,
}

impl AppState {
    /// Creates a new instance of AppState.
    ///
    /// Initializes with provided configuration, combined tool definitions, and merged tool handlers.
    /// Tool definitions are sorted by name for consistent presentation.
    ///
    /// # Arguments
    /// * `config` - Application configuration.
    /// * `all_definitions` - A vector containing all tool definitions (internal and external).
    /// * `merged_handlers` - A map containing all tool handlers (internal and external).
    ///
    /// # Returns
    /// A new `AppState` instance.
    pub fn new(
        config: crate::config::app_config::AppConfig,
        tool_schemas: Option<crate::tools::tools_schema::ToolsSchema>,
        tool_handler: Option<crate::tools::tool_handler::ToolHandler>,
    ) -> Self {
        // Sort definitions by name for consistent presentation to LLM and in documentation.

        Self {
            config,
            sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            ws_connections: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            tool_schemas: std::sync::Arc::new(tool_schemas),
            tool_handler: std::sync::Arc::new(tool_handler),
        }
    }
}
