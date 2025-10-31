//! Provides a query function to delete an item from a user's database collection.
//!
//! This function handles verifying ownership of the parent collection and then
//! deleting the specified item. It returns the number of rows affected (u64)
//! or an error if the operation fails or permissions are denied.
//! Adheres to 'one item per file' and FQN guidelines.

/// Deletes a specific item from a user's database collection after verifying ownership.
///
/// # Arguments
/// * `pool` - A reference to the `sqlx::PgPool` for database connectivity.
/// * `user_id` - The `uuid::Uuid` of the user attempting the deletion, for ownership verification.
/// * `collection_id_uuid` - The `uuid::Uuid` of the parent collection.
/// * `item_id_uuid` - The `uuid::Uuid` of the item to be deleted.
///
/// # Returns
/// * `Result<u64, anyhow::Error>` - `Ok(rows_affected)` on successful deletion,
///   or an `Err` containing an `anyhow::Error` if:
///     - The parent collection is not found.
///     - The user does not own the parent collection.
///     - A database error occurs during fetching or deletion.
pub async fn delete_user_db_collection_item_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_uuid: uuid::Uuid,
    item_id_uuid: uuid::Uuid,
) -> std::result::Result<u64, anyhow::Error> {
    // 1. Verify ownership of parent collection
    let collection_owner_id_result: std::result::Result<Option<uuid::Uuid>, sqlx::Error> = sqlx::query_scalar!(
        r#"SELECT user_id FROM user_db_collections WHERE id = $1"#,
        collection_id_uuid
    )
    .fetch_optional(pool)
    .await;

    match collection_owner_id_result {
        Ok(Some(owner_id)) => {
            if owner_id != user_id {
                return std::result::Result::Err(anyhow::anyhow!("User does not own the parent collection."));
            }
        }
        Ok(None) => {
            return std::result::Result::Err(anyhow::anyhow!("Parent collection not found."));
        }
        Err(e) => {
            return std::result::Result::Err(anyhow::anyhow!("Failed to fetch parent collection: {}", e));
        }
    }

    // 2. Delete item
    let result = sqlx::query!(
        r#"
        DELETE FROM user_db_collection_items
        WHERE id = $1 AND user_db_collection_id = $2
        "#,
        item_id_uuid,
        collection_id_uuid
    )
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to delete collection item from database: {}", e))?;

    std::result::Result::Ok(result.rows_affected())
}
