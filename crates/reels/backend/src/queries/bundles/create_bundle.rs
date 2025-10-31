//! Defines the `create_bundle` database query function.
//!
//! This function inserts a new bundle into the `bundles` table.
//! It takes user ID, name, description, style ID, and IDs for documents,
//! assets, and formats as input. Returns the newly created bundle.
//! Adheres to the project's Rust coding standards.

// Revision History
// - 2025-05-29T18:15:36Z @AI: Initial creation, moved from db/bundles.rs and refactored.

/// Creates a new bundle in the database.
pub async fn create_bundle(
    pool: &sqlx::postgres::PgPool,
    user_id: sqlx::types::Uuid,
    name: &str,
    description: Option<&str>,
    style_id: sqlx::types::Uuid,
    document_ids: &[sqlx::types::Uuid],
    asset_ids: &[sqlx::types::Uuid],
    format_ids: &[sqlx::types::Uuid],
) -> std::result::Result<crate::db::bundles::Bundle, sqlx::Error> {
    let bundle = sqlx::query_as!(
        crate::db::bundles::Bundle,
        r#"
        INSERT INTO bundles (user_id, name, description, style_id, document_ids, asset_ids, format_ids)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
        "#,
        user_id,
        name,
        description,
        style_id,
        document_ids,
        asset_ids,
        format_ids
    )
        .fetch_one(pool)
        .await?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    // Example test structure - actual tests would need a test database setup.
    // #[test]
    // fn test_create_bundle_example() {
    //     // Test logic here, likely involving a mock pool or a test database.
    //     // let result = super::create_bundle(...);
    //     // std::assert!(result.is_ok());
    // }
}