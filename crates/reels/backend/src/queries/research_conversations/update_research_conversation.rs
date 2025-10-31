//! Updates an existing research conversation's state in the database.
//!
//! This function modifies the `last_instruction`, `conversation_state_gcs_uri`,
//! and `status` of a conversation identified by its ID. It returns the updated
//! `ResearchConversation` wrapped in an `Option`.

pub async fn update_research_conversation(
    pool: &sqlx::PgPool,
    conversation_id: uuid::Uuid,
    last_instruction: Option<std::string::String>,
    conversation_state_gcs_uri: Option<std::string::String>,
    status: std::string::String,
) -> Result<Option<crate::db::research_conversation::ResearchConversation>, sqlx::Error> {
    let current_time = chrono::Utc::now();
    sqlx::query_as!(
        crate::db::research_conversation::ResearchConversation,
        r#"
        UPDATE research_conversations
        SET
            last_instruction = $1,
            conversation_state_gcs_uri = $2,
            status = $3,
            updated_at = $4
        WHERE id = $5
        RETURNING id, user_id, document_id, created_at, updated_at, last_instruction, conversation_state_gcs_uri, status
        "#,
        last_instruction,
        conversation_state_gcs_uri,
        status,
        current_time,
        conversation_id
    )
    .fetch_optional(pool)
    .await
}