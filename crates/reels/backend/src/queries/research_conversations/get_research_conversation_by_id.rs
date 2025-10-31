//! Retrieves a single research conversation from the database by its ID.
//!
//! This function queries for a specific `ResearchConversation` and returns
//! it wrapped in an `Option`, which will be `None` if no conversation
//! with the given ID is found.

pub async fn get_research_conversation_by_id(
    pool: &sqlx::PgPool,
    conversation_id: uuid::Uuid,
) -> Result<Option<crate::db::research_conversation::ResearchConversation>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::research_conversation::ResearchConversation,
        r#"
        SELECT id, user_id, document_id, created_at, updated_at, last_instruction, conversation_state_gcs_uri, status
        FROM research_conversations WHERE id = $1
        "#,
        conversation_id
    )
    .fetch_optional(pool)
    .await
}