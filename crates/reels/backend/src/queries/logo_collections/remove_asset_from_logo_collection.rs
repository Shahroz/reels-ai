//! Removes an asset from a logo collection.
//!
//! This function removes the association between an asset and a logo collection.
//! Only removes the association, not the asset itself.
//! Returns whether the removal was successful.

/// Removes an asset from a logo collection
pub async fn remove_asset_from_logo_collection(
    pool: &sqlx::PgPool,
    logo_collection_id: uuid::Uuid,
    asset_id: uuid::Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM logo_collection_assets WHERE logo_collection_id = $1 AND asset_id = $2",
        logo_collection_id,
        asset_id
    )
    .execute(pool)
    .await?;

    std::result::Result::Ok(result.rows_affected() > 0)
}


// Tests temporarily disabled - need proper test infrastructure
