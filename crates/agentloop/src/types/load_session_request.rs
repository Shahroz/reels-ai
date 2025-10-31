//! Defines the request payload for loading a session state.
//!
//! This struct contains the necessary data to reconstruct a session,
//! excluding fields like session_id and timestamps which are generated
//! upon loading. Adheres to project guidelines.

use utoipa::ToSchema; // Import ToSchema for OpenAPI documentation

/// Payload for the POST /session/load endpoint.
///
/// Contains the state needed to recreate a session. The server will generate
/// a new session_id and timestamps upon loading.
#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct LoadSessionRequest {
    pub user_id: uuid::Uuid,
    /// Optional organization ID associated with this session (for organization credit context)
    pub organization_id: std::option::Option<uuid::Uuid>,
    /// Current status of the session (e.g., Pending, InProgress).
    pub status: crate::types::session_status::SessionStatus,
    /// Configuration parameters applied to this session.
    pub config: crate::types::session_config::SessionConfig,
    /// Ordered list of conversation entries (user, agent, tool messages).
    pub history: std::vec::Vec<crate::types::conversation_entry::ConversationEntry>,
    /// Collection of context snippets gathered during the session.
    pub context: std::vec::Vec<crate::types::context_entry::ContextEntry>,
    /// The primary research goal or objective for the session.
    pub research_goal: std::option::Option<std::string::String>,
    /// Optional system message to guide the assistant.
    pub system_message: std::option::Option<std::string::String>,
    /// Chronological list of messages exchanged directly for prompt building.
    pub messages: std::vec::Vec<crate::types::message::Message>,
}

// No tests needed for this simple data structure definition.