//! Defines the `update_bundle` database query function.
//!
//! This function updates an existing bundle in the `bundles` table.
//! It allows for partial updates; fields set to `None` will not be changed.
//! Ensures the user owns the bundle before updating.
//! Adheres to the project's Rust coding standards.

// Revision History
// - 2025-05-29T18:15:36Z @AI: Initial creation, moved from db/bundles.rs and refactored.

/// Updates an existing bundle.
/// Fields set to `None` will not be updated.
#[allow(clippy::too_many_arguments)]
pub async fn update_bundle(
    pool: &sqlx::postgres::PgPool,
    bundle_id: sqlx::types::Uuid,
    user_id: sqlx::types::Uuid, // To ensure user owns the bundle
    name: Option<String>,
    description: Option<Option<String>>, // Option<Option<String>> to allow setting description to NULL
    style_id: Option<sqlx::types::Uuid>,
    document_ids: Option<Vec<sqlx::types::Uuid>>,
    asset_ids: Option<Vec<sqlx::types::Uuid>>,
    format_ids: Option<Vec<sqlx::types::Uuid>>,
) -> std::result::Result<crate::db::bundles::Bundle, sqlx::Error> {
    let bundle = sqlx::query_as!(
        crate::db::bundles::Bundle,
        r#"
        UPDATE bundles
        SET
            name = COALESCE($3, name),
            description = CASE WHEN $4::BOOLEAN THEN $5 ELSE description END,
            style_id = COALESCE($6, style_id),
            document_ids = COALESCE($7, document_ids),
            asset_ids = COALESCE($8, asset_ids),
            format_ids = COALESCE($9, format_ids),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
        "#,
        bundle_id,
        user_id,
        name,
        description.is_some(), 
        description.flatten(), 
        style_id,
        document_ids.as_ref().map(|v| v.as_slice()),
        asset_ids.as_ref().map(|v| v.as_slice()),
        format_ids.as_ref().map(|v| v.as_slice())
    )
        .fetch_one(pool)
        .await?;
    Ok(bundle)
}
