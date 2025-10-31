//! Defines the `delete_asset` database query function.
//!
//! This function deletes an asset from the `assets` table by its ID.
//! Now includes automatic studio cleanup to remove orphaned studio nodes and provenance edges.
//! Adheres to the project's Rust coding standards.

use crate::queries::studio_cleanup::cleanup_studio_relationships;

pub async fn delete_asset(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // 1. Clean up studio relationships first (always)
    let _cleanup_summary = cleanup_studio_relationships(&mut tx, asset_id, "asset").await?;
    
    // 2. Delete the main asset
    let result = sqlx::query!("DELETE FROM assets WHERE id = $1", asset_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(result)
}