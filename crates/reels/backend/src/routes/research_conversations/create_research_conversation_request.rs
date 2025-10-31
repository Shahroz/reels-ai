//! Defines the request data structure for creating a new research conversation.
//!
//! This struct encapsulates the necessary information required to initiate
//! a research conversation, including user and document identifiers, an optional
//! initial instruction, and the starting status.

// No `use` statements allowed per guidelines. Use fully qualified paths.

/// Data required to create a new research conversation.
#[allow(dead_code)] // Fields are read during deserialization
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateResearchConversationRequest {
    #[schema(format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub document_id: uuid::Uuid,
    pub last_instruction: Option<std::string::String>,
    #[schema(example = "pending")]
    pub status: std::string::String, // e.g., 'pending', 'running', 'completed', 'failed'
}
