//! Defines the `find_bundle_by_id` database query function.
//!
//! This function retrieves a bundle from the `bundles` table by its unique ID.
//! Returns an `Option<Bundle>`, with `None` if no bundle is found.
//! Adheres to the project's Rust coding standards.

// Revision History
// - 2025-05-29T18:15:36Z @AI: Initial creation, moved from db/bundles.rs and refactored.

/// Finds a bundle by its unique ID.
pub async fn find_bundle_by_id(
    pool: &sqlx::postgres::PgPool,
    bundle_id: sqlx::types::Uuid,
) -> std::result::Result<Option<crate::db::bundles::Bundle>, sqlx::Error> {
    let bundle = sqlx::query_as!(
        crate::db::bundles::Bundle,
        r#"
        SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
        FROM bundles
        WHERE id = $1
        "#,
        bundle_id
    )
        .fetch_optional(pool)
        .await?;
    Ok(bundle)
}
