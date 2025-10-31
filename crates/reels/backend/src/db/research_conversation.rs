//! Defines the primary data model for a research conversation.
//!
//! This struct represents a research conversation record in the database,
//! linked to a user and a document. It stores metadata and state information
//! about the conversation process.

// No `use` statements allowed per guidelines. Use fully qualified paths.

/// Represents a research conversation linked to a user and a document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ResearchConversation {
    #[schema(format = "uuid", value_type=String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub user_id: uuid::Uuid,
    #[schema(format = "uuid", value_type=String)]
    pub document_id: uuid::Uuid,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_instruction: Option<std::string::String>,
    pub conversation_state_gcs_uri: Option<std::string::String>,
    #[schema(example = "completed")]
    pub status: std::string::String, // e.g., 'pending', 'running', 'completed', 'failed'
}