//! Defines the request data structure for updating a research conversation's state.
//!
//! This struct contains the fields that can be modified for an existing
//! research conversation, such as the last instruction provided, a link
//! to the persisted conversation state, and the current status.

// No `use` statements allowed per guidelines. Use fully qualified paths.

/// Data required to update the state of a research conversation.
#[allow(dead_code)] // Fields are read during deserialization
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateResearchConversationStateRequest {
    #[schema(example = "Analyze the results section.")]
    pub last_instruction: Option<std::string::String>,
    #[schema(example = "gs://bucket-name/path/to/state.json")]
    pub conversation_state_gcs_uri: Option<std::string::String>,
    #[schema(example = "running")]
    pub status: std::string::String,
}
