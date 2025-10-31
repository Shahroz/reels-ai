//! Counts custom creative formats based on search and ownership criteria.
//!
//! This function provides a total count of custom creative formats, filtering by
//! public status or user ownership, and by a name search pattern. It's used
//! for pagination purposes.
//! Adheres to one item per file guideline.

use sqlx::PgPool;
use uuid::Uuid;

pub async fn count_formats(
    pool: &PgPool,
    user_id: Uuid,
    is_public: bool,
    search: Option<String>,
) -> Result<i64, sqlx::Error> {
    let search_pattern = search.map(|s| format!("%{s}%")).unwrap_or_else(|| "%".into());

    let count = if is_public {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM custom_creative_formats WHERE (is_public = TRUE AND user_id IS NULL) AND name ILIKE $1",
            &search_pattern
        )
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM custom_creative_formats WHERE user_id = $1 AND name ILIKE $2",
            user_id,
            &search_pattern
        )
        .fetch_one(pool)
        .await?
    };

    Ok(count.unwrap_or(0))
}