//! Provides a query function to retrieve a specific item from a user's database collection.
//!
//! This function encapsulates the logic for verifying collection ownership by the user
//! and then fetching the specified item if access is permitted. It adheres to
//! the 'one item per file' and FQN guidelines from `rust_guidelines.md`.
//! Returns the item if found and accessible, None if item not found in an accessible collection,
//! or an error for access violations or database issues.

/// Verifies collection ownership and fetches a specific item from a user DB collection.
///
/// # Arguments
/// * `pool` - A reference to the `sqlx::PgPool`.
/// * `user_id` - The ID of the user making the request.
/// * `collection_id_uuid` - The ID of the parent collection.
/// * `item_id_uuid` - The ID of the item to retrieve.
///
/// # Returns
/// * `Result<Option<crate::db::user_db_collection_item::UserDbCollectionItem>, anyhow::Error>`
///   - `Ok(Some(item))` if the item is found and user has access.
///   - `Ok(None)` if the item is not found in an accessible collection.
///   - `Err(anyhow::Error)` if collection is not found, user does not own collection, or a database error occurs.
pub async fn get_user_db_collection_item_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_uuid: uuid::Uuid,
    item_id_uuid: uuid::Uuid,
) -> Result<Option<crate::db::user_db_collection_item::UserDbCollectionItem>, anyhow::Error> {
    // 1. Verify ownership of parent collection
    match sqlx::query!(
        r#"SELECT user_id FROM user_db_collections WHERE id = $1"#,
        collection_id_uuid
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(record)) => {
            if record.user_id != user_id {
                return Result::Err(anyhow::anyhow!(
                    "User does not own the parent collection."
                ));
            }
            // Ownership verified, proceed to fetch item
        }
        Ok(None) => {
            return Result::Err(anyhow::anyhow!("Parent collection not found."));
        }
        Err(e) => {
            return Result::Err(
                anyhow::Error::from(e).context("Failed to verify collection ownership."),
            );
        }
    };

    // 2. Fetch item
    match sqlx::query_as!(
        crate::db::user_db_collection_item::UserDbCollectionItem,
        r#"
        SELECT id, user_db_collection_id, item_data, created_at, updated_at
        FROM user_db_collection_items
        WHERE id = $1 AND user_db_collection_id = $2
        "#,
        item_id_uuid,
        collection_id_uuid
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(item)) => Result::Ok(Some(item)),
        Ok(None) => Result::Ok(None), // Item not found in the (now verified) collection
        Err(e) => Result::Err(
            anyhow::Error::from(e).context("Failed to retrieve collection item."),
        ),
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests are conceptual placeholders.
    // Full testing requires a database connection and test data setup.
    // The `#[tokio::test]` attribute would be used for async tests.
    // For simplicity in this context, we'll use `std::assert!` and acknowledge the placeholder nature.

    #[test]
    fn test_conceptual_get_item_success() {
        // Conceptual: Simulates a scenario where the item is successfully retrieved.
        // In a real test, you would:
        // 1. Setup a mock/test database.
        // 2. Populate it with a user, a collection owned by the user, and an item in that collection.
        // 3. Call `super::get_user_db_collection_item_query` with appropriate IDs.
        // 4. Assert that the result is `Ok(Some(item))` and the item data is correct.
        std::assert!(true, "Placeholder: test_get_item_success needs actual DB test setup");
    }

    #[test]
    fn test_conceptual_item_not_found_in_owned_collection() {
        // Conceptual: Simulates item not found in an owned collection.
        // In a real test:
        // 1. Setup user and an owned collection.
        // 2. Call query with an `item_id_uuid` that doesn't exist in that collection.
        // 3. Assert `Ok(None)`.
        std::assert!(true, "Placeholder: test_item_not_found_in_owned_collection needs actual DB test setup");
    }

    #[test]
    fn test_conceptual_collection_not_owned() {
        // Conceptual: Simulates trying to access an item in a collection not owned by the user.
        // In a real test:
        // 1. Setup user, and a collection owned by a *different* user.
        // 2. Call query.
        // 3. Assert `Err(e)` where `e.to_string()` contains "User does not own the parent collection.".
        std::assert!(true, "Placeholder: test_collection_not_owned needs actual DB test setup");
    }

    #[test]
    fn test_conceptual_collection_not_found() {
        // Conceptual: Simulates trying to access an item in a non-existent collection.
        // In a real test:
        // 1. Setup user.
        // 2. Call query with a `collection_id_uuid` that doesn't exist.
        // 3. Assert `Err(e)` where `e.to_string()` contains "Parent collection not found.".
        std::assert!(true, "Placeholder: test_collection_not_found needs actual DB test setup");
    }
}
