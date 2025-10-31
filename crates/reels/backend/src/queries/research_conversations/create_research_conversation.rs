//! Inserts a new research conversation record into the database.
//!
//! This function takes user ID, document ID, an optional initial instruction,
//! and a status to create a new research conversation. It returns the newly
//! created `ResearchConversation` object.

pub async fn create_research_conversation(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    document_id: uuid::Uuid,
    last_instruction: Option<std::string::String>,
    status: std::string::String,
) -> Result<crate::db::research_conversation::ResearchConversation, sqlx::Error> {
    sqlx::query_as!(
        crate::db::research_conversation::ResearchConversation,
        r#"
        INSERT INTO research_conversations (user_id, document_id, last_instruction, status)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, document_id, created_at, updated_at, last_instruction, conversation_state_gcs_uri, status
        "#,
        user_id,
        document_id,
        last_instruction,
        status
    )
    .fetch_one(pool)
    .await
}