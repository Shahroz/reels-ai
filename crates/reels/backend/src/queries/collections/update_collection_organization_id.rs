//! Defines the query for updating organization_id of an existing collection.
//!
//! This function updates the organization_id of a collection if it's not already set.
//! It adheres to the one-item-per-file and FQN guidelines.

pub async fn update_collection_organization_id(
    pool: &sqlx::PgPool,
    collection_id: uuid::Uuid,
    organization_id: uuid::Uuid,
) -> Result<crate::db::collections::Collection, sqlx::Error> {
    sqlx::query_as!(
        crate::db::collections::Collection,
        r#"
        UPDATE collections 
        SET organization_id = $2, updated_at = NOW()
        WHERE id = $1 AND organization_id IS NULL
        RETURNING id, user_id, organization_id, name, metadata, created_at, updated_at
        "#,
        collection_id,
        organization_id
    )
    .fetch_one(pool)
    .await
}

