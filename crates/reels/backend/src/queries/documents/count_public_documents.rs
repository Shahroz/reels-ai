//! Counts public documents, optionally filtered by a search pattern and task status.
//!
//! This function executes a SQL query to count rows in the `documents` table
//! that are marked as public and match an optional search pattern and task status.
//! It is designed for pagination or displaying total counts for public listings.
//! Returns the count as `i64` or a `sqlx::Error`.

pub async fn count_public_documents(
    pool: &sqlx::PgPool,
    search_pattern: &str,
    is_task_filter: Option<bool>,
) -> std::result::Result<i64, sqlx::Error> {
    let count_result = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM documents WHERE is_public = true AND (title ILIKE $1 OR content ILIKE $1) AND ($2::BOOLEAN IS NULL OR is_task = $2)"#,
        search_pattern,
        is_task_filter
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(count_result.unwrap_or(0))
}