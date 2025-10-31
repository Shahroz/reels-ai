//! Counts documents for a user, optionally filtered by a search pattern.
//!
//! This function executes a SQL query to count rows in the `documents` table
//! matching a given user ID and a search pattern applied to title and content.
//! It is designed to be used for pagination or displaying total counts.
//! Returns the count as `i64` or a `sqlx::Error`.

// No use statements allowed.

pub async fn count_documents_for_user(
    pool: &sqlx::PgPool,
   user_id: uuid::Uuid,
   search_pattern: &str,
) -> std::result::Result<i64, sqlx::Error> {
    type TotalCount = crate::sql_utils::count_sql_results::TotalCount;
    let query_result = sqlx::query_as!(
        TotalCount,
        "SELECT COUNT(*) as count FROM documents WHERE (user_id = $1 OR is_public = true) AND (title ILIKE $2 OR content ILIKE $2)",
        user_id,
        search_pattern
    )
    .fetch_one(pool)
    .await;

    match query_result {
        std::result::Result::Ok(result) => std::result::Result::Ok(result.count.unwrap_or_default()),
        std::result::Result::Err(e) => std::result::Result::Err(e),
    }
}
