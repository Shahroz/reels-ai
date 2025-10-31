//! Retrieves all research conversations associated with a specific document ID.
//!
//! This function fetches a list of `ResearchConversation` objects from the database
//! that are linked to the provided `document_id`. The results are ordered by
//! their creation date in descending order.

pub async fn list_research_conversations_by_document_id(
    pool: &sqlx::PgPool,
    document_id: uuid::Uuid,
) -> Result<Vec<crate::db::research_conversation::ResearchConversation>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::research_conversation::ResearchConversation,
        "SELECT id, user_id, document_id, created_at, updated_at, last_instruction, conversation_state_gcs_uri, status FROM research_conversations WHERE document_id = $1 ORDER BY created_at DESC",
        document_id
    )
    .fetch_all(pool)
    .await
}