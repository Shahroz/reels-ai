//! Deletes a research conversation from the database by its ID.
//!
//! This function executes a DELETE query and returns the number of rows affected.
//! A result of 0 rows affected indicates that the conversation was not found.

pub async fn delete_research_conversation(
    pool: &sqlx::PgPool,
    conversation_id: uuid::Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM research_conversations WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}