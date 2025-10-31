//! Defines the `delete_bundle` database query function.
//!
//! This function deletes a bundle from the `bundles` table by its ID.
//! It ensures that the user attempting the deletion owns the bundle.
//! Returns the number of rows affected (should be 1 if successful).
//! Adheres to the project's Rust coding standards.

// Revision History
// - 2025-05-29T18:15:36Z @AI: Initial creation, moved from db/bundles.rs and refactored.

/// Deletes a bundle by its ID, ensuring the user owns it.
pub async fn delete_bundle(
    pool: &sqlx::postgres::PgPool,
    bundle_id: sqlx::types::Uuid,
    user_id: sqlx::types::Uuid, // To ensure user owns the bundle
) -> std::result::Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM bundles WHERE id = $1 AND user_id = $2",
        bundle_id,
        user_id
    )
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    // Example test structure
    // #[test]
    // fn test_delete_bundle_example() {
    //     // Test logic here
    // }
}