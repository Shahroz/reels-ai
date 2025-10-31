//! Defines the `get_asset_by_id` database query function.
//!
//! This function retrieves a single asset from the `assets` table by its ID.
//! Adheres to the project's Rust coding standards.

pub async fn get_asset_by_id(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
) -> Result<Option<crate::db::assets::Asset>, sqlx::Error> {
    let asset_result = sqlx::query_as!(
        crate::db::assets::Asset,
        "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = $1",
        asset_id
    )
    .fetch_optional(pool)
    .await;

    asset_result
}