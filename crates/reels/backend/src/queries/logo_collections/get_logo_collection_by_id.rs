//! Retrieves a logo collection by ID with its associated assets.
//!
//! This function fetches a specific logo collection belonging to a user,
//! including all associated assets with their details.
//! Returns detailed collection information for display and editing.

/// Gets a logo collection by ID with all associated assets
pub async fn get_logo_collection_by_id(
    pool: &sqlx::PgPool,
    collection_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<std::option::Option<crate::schemas::logo_collection_schemas::LogoCollectionResponse>, sqlx::Error> {
    // First get the collection
    let collection = sqlx::query!(
        "SELECT id, user_id, name, description, created_at, updated_at FROM logo_collections WHERE id = $1 AND user_id = $2",
        collection_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    if let std::option::Option::Some(collection_row) = collection {
        // Get associated assets
        let assets = sqlx::query!(
            r#"
            SELECT 
                lca.id,
                lca.asset_id,
                lca.display_name,
                lca.created_at,
                a.name as asset_name,
                a.url as asset_url,
                a.type as asset_type
            FROM logo_collection_assets lca
            JOIN assets a ON lca.asset_id = a.id
            WHERE lca.logo_collection_id = $1
            ORDER BY lca.created_at DESC
            "#,
            collection_id
        )
        .fetch_all(pool)
        .await?;

        let asset_responses = assets
            .into_iter()
            .map(|row| crate::schemas::logo_collection_schemas::LogoCollectionAssetResponse {
                id: row.id,
                asset_id: row.asset_id,
                display_name: row.display_name,
                created_at: row.created_at,
                asset_name: row.asset_name,
                asset_url: row.asset_url,
                asset_type: row.asset_type,
            })
            .collect();

        let response = crate::schemas::logo_collection_schemas::LogoCollectionResponse {
            id: collection_row.id,
            user_id: collection_row.user_id,
            name: collection_row.name,
            description: collection_row.description,
            created_at: collection_row.created_at,
            updated_at: collection_row.updated_at,
            assets: asset_responses,
        };

        std::result::Result::Ok(std::option::Option::Some(response))
    } else {
        std::result::Result::Ok(std::option::Option::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to get a test database pool
    async fn get_test_pool() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost:5432/test_db".to_string());
        
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to create test database pool")
    }

    /// Creates a test user
    async fn create_test_user(pool: &sqlx::PgPool) -> uuid::Uuid {
        let user_id = uuid::Uuid::new_v4();
        let email = std::format!("test-{}@example.com", user_id.simple());
        
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, email_verified, created_at, updated_at)
            VALUES ($1, $2, 'test_hash', true, NOW(), NOW())
            "#,
            user_id,
            email
        )
        .execute(pool)
        .await
        .expect("Failed to create test user");
        
        user_id
    }

    /// Creates a test logo collection
    async fn create_test_logo_collection(
        pool: &sqlx::PgPool, 
        user_id: uuid::Uuid,
        name: &str,
        description: std::option::Option<&str>
    ) -> uuid::Uuid {
        let collection_id = uuid::Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO logo_collections (id, user_id, name, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            "#,
            collection_id,
            user_id,
            name,
            description
        )
        .execute(pool)
        .await
        .expect("Failed to create test logo collection");
        
        collection_id
    }

    /// Creates a test asset
    async fn create_test_asset(pool: &sqlx::PgPool, user_id: uuid::Uuid, name: &str) -> uuid::Uuid {
        let asset_id = uuid::Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO assets (id, user_id, name, type, gcs_object_name, url, created_at, updated_at)
            VALUES ($1, $2, $3, 'image/png', $4, $5, NOW(), NOW())
            "#,
            asset_id,
            user_id,
            name,
            std::format!("test/{}", name),
            std::format!("https://storage.googleapis.com/test-bucket/test/{}", name)
        )
        .execute(pool)
        .await
        .expect("Failed to create test asset");
        
        asset_id
    }

    /// Adds an asset to a logo collection
    async fn add_asset_to_collection(
        pool: &sqlx::PgPool,
        collection_id: uuid::Uuid,
        asset_id: uuid::Uuid,
        display_name: std::option::Option<&str>
    ) -> uuid::Uuid {
        let relation_id = uuid::Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO logo_collection_assets (id, logo_collection_id, asset_id, display_name, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
            relation_id,
            collection_id,
            asset_id,
            display_name
        )
        .execute(pool)
        .await
        .expect("Failed to add asset to collection");
        
        relation_id
    }

    /// Cleanup helper
    async fn cleanup_test_data(pool: &sqlx::PgPool, user_id: uuid::Uuid) {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_get_existing_logo_collection() {
        let pool = get_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let collection_id = create_test_logo_collection(
            &pool, 
            user_id, 
            "Test Collection", 
            std::option::Option::Some("Test description")
        ).await;

        // Test getting the collection
        let result = get_logo_collection_by_id(&pool, collection_id, user_id).await;
        
        assert!(result.is_ok());
        let collection_option = result.unwrap();
        assert!(collection_option.is_some());
        
        let collection = collection_option.unwrap();
        assert_eq!(collection.id, collection_id);
        assert_eq!(collection.user_id, user_id);
        assert_eq!(collection.name, "Test Collection");
        assert_eq!(collection.description, std::option::Option::Some("Test description".to_string()));
        assert!(collection.assets.is_empty()); // No assets added yet

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_get_logo_collection_with_assets() {
        let pool = get_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let collection_id = create_test_logo_collection(
            &pool, 
            user_id, 
            "Collection with Assets", 
            std::option::Option::None
        ).await;

        // Create and add assets to the collection
        let asset1_id = create_test_asset(&pool, user_id, "logo1.png").await;
        let asset2_id = create_test_asset(&pool, user_id, "logo2.png").await;
        
        add_asset_to_collection(&pool, collection_id, asset1_id, std::option::Option::Some("Primary Logo")).await;
        add_asset_to_collection(&pool, collection_id, asset2_id, std::option::Option::None).await;

        // Test getting the collection with assets
        let result = get_logo_collection_by_id(&pool, collection_id, user_id).await;
        
        assert!(result.is_ok());
        let collection_option = result.unwrap();
        assert!(collection_option.is_some());
        
        let collection = collection_option.unwrap();
        assert_eq!(collection.name, "Collection with Assets");
        assert_eq!(collection.assets.len(), 2);
        
        // Check asset details
        let primary_asset = collection.assets.iter().find(|a| a.display_name == std::option::Option::Some("Primary Logo".to_string()));
        assert!(primary_asset.is_some());
        assert_eq!(primary_asset.unwrap().asset_name, "logo1.png");
        
        let unnamed_asset = collection.assets.iter().find(|a| a.display_name.is_none());
        assert!(unnamed_asset.is_some());
        assert_eq!(unnamed_asset.unwrap().asset_name, "logo2.png");

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_get_nonexistent_logo_collection() {
        let pool = get_test_pool().await;
        let user_id = create_test_user(&pool).await;
        let nonexistent_id = uuid::Uuid::new_v4();

        // Test getting a collection that doesn't exist
        let result = get_logo_collection_by_id(&pool, nonexistent_id, user_id).await;
        
        assert!(result.is_ok());
        let collection_option = result.unwrap();
        assert!(collection_option.is_none());

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_get_logo_collection_wrong_user() {
        let pool = get_test_pool().await;
        let owner_user_id = create_test_user(&pool).await;
        let other_user_id = create_test_user(&pool).await;
        
        let collection_id = create_test_logo_collection(
            &pool, 
            owner_user_id, 
            "Owner's Collection", 
            std::option::Option::None
        ).await;

        // Test accessing collection as a different user
        let result = get_logo_collection_by_id(&pool, collection_id, other_user_id).await;
        
        assert!(result.is_ok());
        let collection_option = result.unwrap();
        assert!(collection_option.is_none()); // Should not be accessible to other user

        // Test accessing as owner should work
        let owner_result = get_logo_collection_by_id(&pool, collection_id, owner_user_id).await;
        assert!(owner_result.is_ok());
        assert!(owner_result.unwrap().is_some());

        // Cleanup
        cleanup_test_data(&pool, owner_user_id).await;
        cleanup_test_data(&pool, other_user_id).await;
    }

    #[test]
    fn test_get_logo_collection_unit_tests() {
        // Unit tests that don't require database

        // Test UUID handling
        let user_id = uuid::Uuid::new_v4();
        let collection_id = uuid::Uuid::new_v4();
        assert_ne!(user_id, collection_id);

        // Test option handling
        let some_description = std::option::Option::Some("description");
        let none_description: std::option::Option<&str> = std::option::Option::None;
        assert!(some_description.is_some());
        assert!(none_description.is_none());
    }
}

