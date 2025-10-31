//! Defines the `update_asset_collection` database query function.
//!
//! This function updates the collection_id of an asset in the `assets` table.
//! It takes the asset ID and the new collection ID (which can be None to remove the association).
//! Adheres to the project's Rust coding standards.

pub async fn update_asset_collection(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
    collection_id: Option<uuid::Uuid>,
) -> Result<crate::db::assets::Asset, sqlx::Error> {
    let result = sqlx::query_as!(
        crate::db::assets::Asset,
        r#"
        UPDATE assets 
        SET collection_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public
        "#,
        asset_id,
        collection_id
    )
    .fetch_one(pool)
    .await;
    result
} 