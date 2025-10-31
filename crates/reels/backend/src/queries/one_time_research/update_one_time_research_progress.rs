//! Appends a progress entry to a one-time research task's progress log.
//!
//! This function takes a database pool, a research task ID, and a JSON value
//! representing a single progress update. It uses the PostgreSQL `||` operator
//! to append the new entry to the `progress_log` JSONB array column.
//! This ensures that progress updates are stored chronologically.

#[tracing::instrument(skip(pool, progress_entry))]
pub async fn update_one_time_research_progress(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    progress_entry: &serde_json::Value,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE one_time_researches
        SET progress_log = progress_log || $1::jsonb
        WHERE id = $2
        "#,
        progress_entry,
        id
    )
    .execute(pool)
    .await?;
    std::result::Result::Ok(())
}