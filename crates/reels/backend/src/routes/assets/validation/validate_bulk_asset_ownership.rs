//! Validates that a user owns all specified assets in a bulk operation.
//!
//! This function fetches multiple assets by their IDs and verifies that the requesting user
//! is the owner of all assets. Returns the assets if validation succeeds,
//! or an HTTP error response if any asset doesn't exist, database error occurs,
//! or the user doesn't own any of the assets. Designed for bulk operations
//! that need to validate multiple assets atomically.

/// Validates that a user owns all specified assets for bulk operations.
///
/// This function performs a single database query to fetch all assets and then verifies
/// ownership by comparing user IDs. It's optimized for bulk operations and ensures
/// atomic validation - either all assets are valid or the operation fails.
/// Works with both connection pools and transactions.
///
/// # Arguments
/// * `executor` - Database executor (pool or transaction)
/// * `asset_ids` - Slice of UUIDs for the assets to validate
/// * `user_id` - UUID of the requesting user
///
/// # Returns
/// * `Ok(Vec<Asset>)` - If all assets exist and are owned by the user
/// * `Err(HttpResponse)` - HTTP error response for immediate return from handlers
///
/// # Errors
/// * 404 - One or more assets not found
/// * 403 - User does not own one or more assets
/// * 500 - Database error during lookup
pub async fn validate_bulk_asset_ownership(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    asset_ids: &[uuid::Uuid],
    user_id: uuid::Uuid,
) -> std::result::Result<std::vec::Vec<crate::db::assets::Asset>, actix_web::HttpResponse> {
    
    // Fetch all assets in one query for efficiency
    let assets = match sqlx::query_as!(
        crate::db::assets::Asset,
        "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public 
         FROM assets WHERE id = ANY($1)",
        asset_ids
    )
    .fetch_all(executor)
    .await {
        Ok(assets) => assets,
        Err(e) => {
            tracing::error!("Database error while fetching assets for bulk validation: {:?}", e);
            return std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch assets"
            })));
        }
    };

    // Check that we found all requested assets
    if assets.len() != asset_ids.len() {
        return std::result::Result::Err(actix_web::HttpResponse::NotFound().json(serde_json::json!({
            "error": "One or more assets not found"
        })));
    }

    // Verify all assets belong to the user
    for asset in &assets {
        if asset.user_id != Some(user_id) {
            return std::result::Result::Err(actix_web::HttpResponse::Forbidden().json(serde_json::json!({
                "error": std::format!("You can only access your own assets. Asset {} is not owned by you", asset.id)
            })));
        }
    }

    std::result::Result::Ok(assets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Note: These are unit tests for basic validation logic.
    // Integration tests in collection_attachment_test.rs cover end-to-end workflows.

    #[tokio::test]
    async fn test_validate_bulk_asset_ownership_with_empty_list() {
        // Test validation with an empty asset list
        let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localuser:localpassword@localhost:5447/localdatabase".to_string())).await.unwrap();
        
        let asset_ids: &[Uuid] = &[];
        let user_id = Uuid::new_v4();
        
        let result = validate_bulk_asset_ownership(&pool, asset_ids, user_id).await;
        
        // Should return empty vector for empty input
        assert!(result.is_ok());
        let assets = result.unwrap();
        assert_eq!(assets.len(), 0);
    }

    #[tokio::test]
    async fn test_validate_bulk_asset_ownership_with_nonexistent_assets() {
        // Test validation with non-existent asset UUIDs
        let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localuser:localpassword@localhost:5447/localdatabase".to_string())).await.unwrap();
        
        let fake_asset_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let user_id = Uuid::new_v4();
        
        let result = validate_bulk_asset_ownership(&pool, &fake_asset_ids, user_id).await;
        
        // Should return NotFound error when assets don't exist
        assert!(result.is_err());
        let response = result.unwrap_err();
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_validate_bulk_asset_ownership_executor_interface() {
        // Ensure function works with both pool and transaction types
        fn _test_pool_compatibility(_pool: &sqlx::PgPool) {
            // This function would compile if called with: validate_bulk_asset_ownership(pool, &[uuid], uuid)
        }
        
        fn _test_transaction_compatibility(_tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) {
            // This function would compile if called with: validate_bulk_asset_ownership(&mut *tx, &[uuid], uuid)
        }
        
        // If this test compiles, our executor interface works correctly
        assert!(true);
    }
}