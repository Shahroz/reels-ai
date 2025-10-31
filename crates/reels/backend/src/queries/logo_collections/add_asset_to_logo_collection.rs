//! Adds an asset to a logo collection.
//!
//! This function creates a new logo_collection_assets record linking an asset
//! to a logo collection. It includes optional display name functionality
//! and prevents duplicate associations through unique constraints.

/// Adds an asset to a logo collection with optional display name
pub async fn add_asset_to_logo_collection(
    pool: &sqlx::PgPool,
    logo_collection_id: uuid::Uuid,
    asset_id: uuid::Uuid,
    display_name: std::option::Option<&str>,
) -> std::result::Result<crate::db::logo_collection_asset::LogoCollectionAsset, sqlx::Error> {
    let collection_asset = sqlx::query_as!(
        crate::db::logo_collection_asset::LogoCollectionAsset,
        r#"
        INSERT INTO logo_collection_assets (logo_collection_id, asset_id, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, logo_collection_id, asset_id, display_name, created_at
        "#,
        logo_collection_id,
        asset_id,
        display_name
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(collection_asset)
}

/// Checks if an asset is already in a logo collection
pub async fn is_asset_in_logo_collection(
    pool: &sqlx::PgPool,
    logo_collection_id: uuid::Uuid,
    asset_id: uuid::Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM logo_collection_assets
        WHERE logo_collection_id = $1 AND asset_id = $2
        "#,
        logo_collection_id,
        asset_id
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(count.count.unwrap_or(0) > 0)
}


// Tests temporarily disabled - need proper test infrastructure
