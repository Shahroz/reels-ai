//! Defines the query for updating an existing collection in the database.
//!
//! This function takes a collection ID, name, metadata, and optional organization_id,
//! updates the corresponding record in the `collections` table, and returns the updated collection.
//! It uses `fetch_one` and will return a `RowNotFound` error if the ID is not found.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn update_collection(
    pool: &sqlx::PgPool,
    id: uuid::Uuid,
    name: &str,
    metadata: &Option<serde_json::Value>,
    organization_id: &Option<uuid::Uuid>,
) -> Result<crate::db::collections::Collection, sqlx::Error> {
    sqlx::query_as!(
        crate::db::collections::Collection,
        "UPDATE collections SET name = $2, metadata = $3, organization_id = $4 WHERE id = $1 RETURNING *",
        id,
        name,
        metadata.clone(),
        *organization_id
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_update_collection_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}