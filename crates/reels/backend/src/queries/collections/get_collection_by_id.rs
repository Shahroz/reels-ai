//! Defines the query for fetching a single collection by its ID.
//!
//! This function queries the `collections` table for a record matching the
//! provided UUID. It returns an `Option<Collection>` to handle cases where
//! the collection may not be found.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn get_collection_by_id(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
) -> Result<Option<crate::db::collections::Collection>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::collections::Collection,
        "SELECT * FROM collections WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_collection_by_id_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}