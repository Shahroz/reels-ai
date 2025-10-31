//! Deletes a logo collection and its associated assets.
//!
//! This function removes a logo collection and all its asset associations.
//! Only the collection owner can delete their collections.
//! Uses CASCADE DELETE to automatically remove asset associations.

/// Deletes a logo collection by ID
pub async fn delete_logo_collection(
    pool: &sqlx::PgPool,
    collection_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM logo_collections WHERE id = $1 AND user_id = $2",
        collection_id,
        user_id
    )
    .execute(pool)
    .await?;

    std::result::Result::Ok(result.rows_affected() > 0)
}


// Tests temporarily disabled - need proper test infrastructure
