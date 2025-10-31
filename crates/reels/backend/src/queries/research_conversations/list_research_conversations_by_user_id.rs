//! Retrieves a paginated list of research conversations for a specific user.
//!
//! This function can filter conversations by status and supports pagination
//! through limit and offset parameters. Results are ordered by creation date
//! in descending order.

pub async fn list_research_conversations_by_user_id(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    status_filter: Option<std::string::String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::db::research_conversation::ResearchConversation>, sqlx::Error> {
    match status_filter {
        Some(status) => {
            sqlx::query_as!(
                crate::db::research_conversation::ResearchConversation,
                r#"
                SELECT id, user_id, document_id, created_at, updated_at, last_instruction, conversation_state_gcs_uri, status
                FROM research_conversations
                WHERE user_id = $1 AND status = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                user_id,
                status,
                limit,
                offset
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                crate::db::research_conversation::ResearchConversation,
                r#"
                SELECT id, user_id, document_id, created_at, updated_at, last_instruction, conversation_state_gcs_uri, status
                FROM research_conversations
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                user_id,
                limit,
                offset
            )
            .fetch_all(pool)
            .await
        }
    }
}