//! Defines the query for creating a new collection in the database.
//!
//! This function takes user ID, name, and metadata, inserts a new record
//! into the `collections` table, and returns the newly created collection.
//! It adheres to the one-item-per-file and FQN guidelines.

pub async fn create_collection(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    name: &str,
    metadata: &Option<serde_json::Value>,
    organization_id: &Option<uuid::Uuid>,
) -> Result<crate::db::collections::Collection, sqlx::Error> {
    sqlx::query_as!(
        crate::db::collections::Collection,
        r#"
        INSERT INTO collections (user_id, name, metadata, organization_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, organization_id, name, metadata, created_at, updated_at
        "#,
        user_id,
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
    fn test_create_collection_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}