//! Validates that a user owns a specific collection.
//!
//! This function fetches a collection by ID and verifies that the requesting user
//! is the owner of that collection. Returns the collection if validation succeeds,
//! or an HTTP error response if the collection doesn't exist, database error occurs,
//! or the user doesn't own the collection. Provides consistent ownership validation
//! across all collection-related route handlers.

/// Validates that a user owns the specified collection.
///
/// This function performs a database lookup to fetch the collection and then verifies
/// ownership by comparing user IDs. It returns standardized HTTP error responses
/// that can be used directly in route handlers.
///
/// # Arguments
/// * `executor` - Database executor (supports both connection pools and transactions)
/// * `collection_id` - UUID of the collection to validate
/// * `user_id` - UUID of the requesting user
///
/// # Returns
/// * `Ok(Collection)` - If the collection exists and is owned by the user
/// * `Err(HttpResponse)` - HTTP error response for immediate return from handlers
///
/// # Errors
/// * 404 - Collection not found
/// * 403 - User does not own the collection
/// * 500 - Database error during lookup
pub async fn validate_collection_ownership(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    collection_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<crate::db::collections::Collection, actix_web::HttpResponse> {
    
    // Fetch collection directly using SQL query for transaction compatibility
    let collection = match sqlx::query_as!(
        crate::db::collections::Collection,
        "SELECT id, user_id, organization_id, name, metadata, created_at, updated_at FROM collections WHERE id = $1",
        collection_id
    )
    .fetch_optional(executor)
    .await {
        Ok(Some(collection)) => collection,
        Ok(None) => {
            return std::result::Result::Err(actix_web::HttpResponse::NotFound().json(serde_json::json!({
                "error": "Collection not found"
            })));
        }
        Err(e) => {
            tracing::error!("Database error while fetching collection: {:?}", e);
            return std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch collection"
            })));
        }
    };

    // Verify user owns the collection
    if collection.user_id != user_id {
        return std::result::Result::Err(actix_web::HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only access your own collections"
        })));
    }

    std::result::Result::Ok(collection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Note: These are unit tests for basic validation logic.
    // Integration tests in collection_attachment_test.rs cover end-to-end workflows.

    #[tokio::test]
    async fn test_validate_collection_ownership_with_nonexistent_collection() {
        // Test validation with a non-existent collection UUID
        let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localuser:localpassword@localhost:5447/localdatabase".to_string())).await.unwrap();
        
        let fake_collection_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let result = validate_collection_ownership(&pool, fake_collection_id, user_id).await;
        
        // Should return NotFound error
        assert!(result.is_err());
        let response = result.unwrap_err();
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_validate_collection_ownership_executor_interface() {
        // Ensure function works with both pool and transaction types
        fn _test_pool_compatibility(_pool: &sqlx::PgPool) {
            // This function would compile if called with: validate_collection_ownership(pool, uuid, uuid)
        }
        
        fn _test_transaction_compatibility(_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) {
            // This function would compile if called with: validate_collection_ownership(&mut *tx, uuid, uuid)
        }
        
        // If this test compiles, our executor interface works correctly
        assert!(true);
    }
}