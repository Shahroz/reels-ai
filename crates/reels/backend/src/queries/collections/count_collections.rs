//! Defines the query for counting collections for a user.
//!
//! This function counts the total number of collections belonging to a specific user,
//! with an optional search filter applied to the collection name.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn count_collections(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM collections WHERE user_id = $1 AND name ILIKE $2",
        user_id,
        search_pattern
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_count_collections_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}