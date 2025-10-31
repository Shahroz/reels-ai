//! Defines the in-memory representation of a research session.
//!
//! This structure encapsulates all data related to a single user interaction session,
//! including configuration, status, conversation history, gathered context,
//! messages, and timestamps for tracking session lifetime and activity.
//! Updated to include system_message and messages fields, and adhere to FQP guidelines.

// Note: Adhering to strict FQP guidelines; removed previous 'use' statements.

/// In-memory representation of a research session.
///
/// Holds the current state, configuration, history, context, messages,
/// creation time, and last activity time for an ongoing research session.
#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)] // Added ToSchema
pub struct SessionData {    /// Unique identifier for the session.
    pub user_id: uuid::Uuid,
    /// Optional organization ID associated with this session (for organization credit context)
    pub organization_id: std::option::Option<uuid::Uuid>,
    pub session_id: std::string::String,
    /// Current status of the session (e.g., Pending, InProgress).
    /// The primary research goal or objective for the session. Updated by user messages.
    pub research_goal: Option<std::string::String>, // FQP for Option and String
    pub status: crate::types::session_status::SessionStatus, // Already FQP-like
    /// Configuration parameters applied to this session.
    pub config: crate::types::session_config::SessionConfig, // Already FQP-like
    /// Ordered list of conversation entries (user, agent, tool messages).
    pub history: std::vec::Vec<crate::types::conversation_entry::ConversationEntry>, // FQP for Vec and element type
    /// Collection of context snippets gathered during the session.
    pub context: std::vec::Vec<crate::types::context_entry::ContextEntry>,    /// The primary research goal or objective for the session.
    /// Timestamp indicating when the session was initiated.
    pub created_at: chrono::DateTime<chrono::Utc>, // Assumes chrono is correctly linked
    /// Timestamp indicating the last recorded activity within the session.
    pub last_activity_timestamp: chrono::DateTime<chrono::Utc>, // Assumes chrono is correctly linked
    /// Optional system message to guide the assistant.
    pub system_message: std::option::Option<std::string::String>, // FQP for Option and String
    /// Chronological list of messages exchanged directly for prompt building.
    pub messages: std::vec::Vec<crate::types::message::Message>, // FQP for Vec and Message type
}


impl SessionData {
    // This constructor is designed to match the signature of calls found in test code.
    // The parameters `_llm_client_mock`, `_timeout_duration_seconds`, and `_max_messages_in_context`
    // are included to satisfy these calls but are currently not used to initialize SessionData fields directly.
    // A more robust implementation might use timeout/max_messages for `config` initialization.
    #[allow(clippy::too_many_arguments)] // To accommodate the 5 arguments from test calls.
    pub fn new(
        user_id: uuid::Uuid,
        session_id_uuid: uuid::Uuid,
        _llm_client_mock: std::sync::Arc<tokio::sync::Mutex<std::option::Option<()>>>, // Ignored parameter
        _timeout_duration_seconds: u64, // Currently unused, placeholder from test call
        _max_messages_in_context: usize, // Currently unused, placeholder from test call
        organization_id: std::option::Option<uuid::Uuid>, // Optional organization ID for credit context
    ) -> Self {
        // Assume SessionConfig and SessionStatus implement Default.
        // If not, specific constructors or enum variants (e.g., SessionStatus::Pending) would be needed.
        let config = crate::types::session_config::SessionConfig::default();
        let status = crate::types::session_status::SessionStatus::default();

        Self {
            user_id,
            organization_id,
            session_id: session_id_uuid.to_string(), // Convert Uuid to String
            research_goal: std::option::Option::None,
            status,
            config,
            history: std::vec::Vec::new(),
            context: std::vec::Vec::new(),
            created_at: chrono::Utc::now(),
            last_activity_timestamp: chrono::Utc::now(),
            system_message: std::option::Option::None,
            messages: std::vec::Vec::new(),
        }
    }
}