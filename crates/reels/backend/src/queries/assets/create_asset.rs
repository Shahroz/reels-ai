//! Defines the `create_asset` database query function.
//!
//! This function inserts a new asset into the `assets` table.
//! It takes all necessary asset details and returns the newly created asset.
//! Adheres to the project's Rust coding standards.

pub async fn create_asset(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
    user_id: Option<uuid::Uuid>,
    name: &str,
    r#type: &str,
    gcs_object_name: &str,
    url: &str,
    collection_id: Option<uuid::Uuid>,
    metadata: Option<serde_json::Value>,
    is_public: bool,
) -> Result<crate::db::assets::Asset, sqlx::Error> {
    let result = sqlx::query_as!(
        crate::db::assets::Asset,
        r#"
        INSERT INTO assets (id, user_id, name, type, gcs_object_name, url, collection_id, metadata, is_public)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public
        "#,
        asset_id,
        user_id,
        name,
        r#type,
        gcs_object_name,
        url,
        collection_id,
        metadata,
        is_public
    )
    .fetch_one(pool)
    .await;
    result
}