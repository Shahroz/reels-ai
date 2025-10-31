#![allow(clippy::disallowed_methods)]
//! Provides a query functiOon to list items within a user DB collection with pagination, sorting, and search.
//!
//! This function encapsulates the database logic for verifying collection ownership,
//! counting total matching items, and fetching a paginated list of items
//! based on the provided criteria.
//! Adheres to 'one item per file' and FQN guidelines.

//! Revision History
//! - 2025-05-09T15:15:29Z @AI: Initial implementation extracted from list_user_db_collection_items route handler.

use anyhow::anyhow;

/// Performs the database query to list user DB collection items.
///
/// Verifies ownership, counts, and fetches items based on pagination, sort, and search criteria.
/// Returns a tuple of (items, total_count) on success, or an `anyhow::Error` on failure.
/// Specific errors for 'Not Found' or 'Forbidden' are indicated by the error message
/// in the returned `anyhow::Error` (e.g., "NotFound: Parent collection not found.").
pub async fn list_user_db_collection_items_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_uuid: uuid::Uuid,
    page: i64, // Assumed to be >= 1, validated by handler
    limit: i64, // Assumed to be >= 1, validated by handler
    sort_by_column_name: &str, // Validated column name by handler
    sort_order: &str, // "ASC" or "DESC", validated by handler
    search_pattern: Option<&str>, // SQL ILIKE pattern e.g., "%term%", prepared by handler
) -> anyhow::Result<(std::vec::Vec<crate::db::user_db_collection_item::UserDbCollectionItem>, i64)> {
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
                return std::result::Result::Err(anyhow::anyhow!("Forbidden: User does not own the parent collection."));
            }
        }
        Ok(None) => {
            return std::result::Result::Err(anyhow::anyhow!("NotFound: Parent collection not found."));
        }
        Err(e) => {
            log::error!("Failed to fetch parent collection for ownership check (query): {e:?}");
            return Err(anyhow::Error::from(e).context("Failed to verify collection ownership during query"));
        }
    };

    if page <= 0 {
        log::error!("Page cannot be 0 or less, it is counted from 1");
        return Err(anyhow!("Page cannot be 0, it is counted from 1"));
    }
    
    let offset = (page - 1) * limit;

    // 2. Execute total count query
    let total_count: i64 = {
        let count_result = match search_pattern {
            Some(pattern) => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM user_db_collection_items WHERE user_db_collection_id = $1 AND item_data::text ILIKE $2",
                    collection_id_uuid,
                    pattern
                )
                .fetch_one(pool)
                .await
            }
            None => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM user_db_collection_items WHERE user_db_collection_id = $1",
                    collection_id_uuid
                )
                .fetch_one(pool)
                .await
            }
        };

        match count_result {
            Ok(Some(count)) => count,
            Ok(None) => 0,
            Err(e) => {
                log::error!(
                    "Failed to count user DB collection items for collection {collection_id_uuid} (query): {e:?}"
                );
                return std::result::Result::Err(
                    anyhow::Error::from(e).context("Failed to count collection items during query"),
                );
            }
        }
    };

    // 3. Execute items fetch query
    let items = {
        // Sanitize sort order and determine sort expression to prevent SQL injection.
        let sanitized_sort_order = if sort_order.to_uppercase() == "DESC" { "DESC" } else { "ASC" };

        let sort_expression = match sort_by_column_name {
            "id" | "created_at" | "updated_at" => sort_by_column_name.to_string(),
            other => {
                if other.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    format!("item_data->>'{other}'")
                } else {
                    return std::result::Result::Err(anyhow::anyhow!(
                        "Invalid sort column name '{}' provided. Only alphanumeric and underscore characters are allowed for JSON fields.",
                        other
                    ));
                }
            }
        };

        let query_result = match search_pattern {
            Some(pattern) => {
                let query_string = format!(
                    "SELECT id, user_db_collection_id, item_data, created_at, updated_at 
                     FROM user_db_collection_items 
                     WHERE user_db_collection_id = $1 AND item_data::text ILIKE $2 
                     ORDER BY {sort_expression} {sanitized_sort_order} LIMIT $3 OFFSET $4"
                );
                sqlx::query_as::<_, crate::db::user_db_collection_item::UserDbCollectionItem>(&query_string)
                    .bind(collection_id_uuid)
                    .bind(pattern)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await
            }
            None => {
                let query_string = format!(
                    "SELECT id, user_db_collection_id, item_data, created_at, updated_at 
                     FROM user_db_collection_items 
                     WHERE user_db_collection_id = $1 
                     ORDER BY {sort_expression} {sanitized_sort_order} LIMIT $2 OFFSET $3"
                );
                sqlx::query_as::<_, crate::db::user_db_collection_item::UserDbCollectionItem>(&query_string)
                    .bind(collection_id_uuid)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await
            }
        };

        match query_result {
            Ok(items) => items,
            Err(e) => {
                log::error!("Failed to list user DB collection items for collection {collection_id_uuid} (query): {e:?}");
                return std::result::Result::Err(
                    anyhow::Error::from(e).context("Failed to retrieve collection items during query"),
                )
            }
        }
    };

    std::result::Result::Ok((items, total_count))
}

#[cfg(test)]
mod tests {
    // Note: These tests require a running database and test data setup.
    // Full test setup is beyond this scope; placeholders illustrate structure.
    // Ensure `DATABASE_URL` is set for `sqlx::test`.
    // Add `test-utils` feature to `sqlx` if using `#[sqlx::test]`.

    // Mock or simplified test setup for demonstration.
    // A real test suite would use `sqlx::test` and a test database.
    fn create_mock_uuid() -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }
    
    // Simulating a very basic in-memory pool for compilation, not execution.
    // This is NOT a functional mock for sqlx::PgPool.
    // For actual tests, a real PgPool connected to a test DB is needed.
    struct MockPool;
    impl MockPool {
        //This is not how you'd actually mock PgPool, but for type checking:
        #[allow(dead_code)]
        fn get_ref(&self) -> &Self { self }
    }


    #[tokio::test]
    #[ignore] // Ignored: Requires a real database connection and test data.
    async fn test_list_items_query_success_structure() {
        // This test is a structural placeholder.
        // To run, replace MockPool with a real sqlx::PgPool connected to a test DB,
        // populate data, and make assertions.
        // let pool = MockPool; // Replace with real pool setup
        let _user_id = create_mock_uuid();
        let _collection_id = create_mock_uuid();

        // Example call structure (won't run without real pool & data)
        // let result = super::list_user_db_collection_items_query(
        //     pool.get_ref(), // This would be &actual_pool
        //     _user_id,
        //     _collection_id,
        //     1, 10, "created_at", "DESC", Some("%")
        // ).await;
        
        // Placeholder assertion
        std::assert!(true, "Test structure for list_items_query_success. Needs DB.");

        // Example assertions if it ran:
        // assert!(result.is_ok());
        // let (items, total_count) = result.unwrap();
        // assert_eq!(total_count, /* expected count from test data */);
        // assert_eq!(items.len(), /* expected item length from test data */);
    }

    #[tokio::test]
    #[ignore] // Ignored: Requires a real database connection.
    async fn test_query_error_handling_structure() {
        // This test is a structural placeholder for error paths.
        // let pool = MockPool; // Replace with real pool setup
        let _user_id = create_mock_uuid();
        // Use a collection_id known to cause an error (e.g., not found, or not owned by user_id)
        let _error_causing_collection_id = create_mock_uuid(); 

        // let result = super::list_user_db_collection_items_query(
        //     pool.get_ref(),
        //     _user_id,
        //     _error_causing_collection_id, // This should trigger an error path
        //     1, 10, "created_at", "DESC", None
        // ).await;

        std::assert!(true, "Test structure for query_error_handling. Needs DB.");
        // Example assertions if it ran:
        // assert!(result.is_err());
        // if let Err(e) = result {
        //     // Check for specific error messages if applicable
        //     // e.g., assert!(e.to_string().contains("NotFound:"));
        // }
    }
}
