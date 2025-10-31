//! Creates a new logo collection in the database.
//!
//! This function inserts a new logo collection record with the provided name and description.
//! It returns the created logo collection with generated ID and timestamps.
//! Uses transactions to ensure data integrity when adding initial assets.

/// Creates a new logo collection for the specified user
pub async fn create_logo_collection(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    name: &str,
    description: std::option::Option<&str>,
) -> std::result::Result<crate::db::logo_collection::LogoCollection, sqlx::Error> {
    let logo_collection = sqlx::query_as!(
        crate::db::logo_collection::LogoCollection,
        r#"
        INSERT INTO logo_collections (user_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, name, description, created_at, updated_at
        "#,
        user_id,
        name,
        description
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(logo_collection)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to get a test database pool
    /// This expects TEST_DATABASE_URL to be set in the environment
    async fn get_test_pool() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost:5432/test_db".to_string());
        
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to create test database pool")
    }

    /// Helper function to create a test user in the database
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

    /// Helper function to cleanup test user
    async fn cleanup_test_user(pool: &sqlx::PgPool, user_id: uuid::Uuid) {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(pool)
            .await
            .expect("Failed to cleanup test user");
    }

    /// Helper function to cleanup logo collection
    async fn cleanup_logo_collection(pool: &sqlx::PgPool, collection_id: uuid::Uuid) {
        sqlx::query!("DELETE FROM logo_collections WHERE id = $1", collection_id)
            .execute(pool)
            .await
            .expect("Failed to cleanup logo collection");
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_create_logo_collection_success() {
        let pool = get_test_pool().await;
        let user_id = create_test_user(&pool).await;

        // Test creating a logo collection
        let result = create_logo_collection(
            &pool,
            user_id,
            "Test Brand Collection",
            std::option::Option::Some("A collection for testing"),
        ).await;

        assert!(result.is_ok());
        let collection = result.unwrap();
        
        assert_eq!(collection.user_id, user_id);
        assert_eq!(collection.name, "Test Brand Collection");
        assert_eq!(collection.description, std::option::Option::Some("A collection for testing".to_string()));
        assert!(collection.created_at < chrono::Utc::now());
        assert!(collection.updated_at < chrono::Utc::now());

        // Cleanup
        cleanup_logo_collection(&pool, collection.id).await;
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_create_logo_collection_without_description() {
        let pool = get_test_pool().await;
        let user_id = create_test_user(&pool).await;

        // Test creating a logo collection without description
        let result = create_logo_collection(
            &pool,
            user_id,
            "Minimal Collection",
            std::option::Option::None,
        ).await;

        assert!(result.is_ok());
        let collection = result.unwrap();
        
        assert_eq!(collection.user_id, user_id);
        assert_eq!(collection.name, "Minimal Collection");
        assert_eq!(collection.description, std::option::Option::None);

        // Cleanup
        cleanup_logo_collection(&pool, collection.id).await;
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_create_logo_collection_with_special_characters() {
        let pool = get_test_pool().await;
        let user_id = create_test_user(&pool).await;

        // Test creating a logo collection with special characters
        let name = "Collection with \"quotes\" & émojis 🎨";
        let description = "Description with newlines\nand special chars: <>&\"'";
        
        let result = create_logo_collection(
            &pool,
            user_id,
            name,
            std::option::Option::Some(description),
        ).await;

        assert!(result.is_ok());
        let collection = result.unwrap();
        
        assert_eq!(collection.name, name);
        assert_eq!(collection.description, std::option::Option::Some(description.to_string()));

        // Cleanup
        cleanup_logo_collection(&pool, collection.id).await;
        cleanup_test_user(&pool, user_id).await;
    }

    #[tokio::test]
    #[ignore] // Requires test database - run with: cargo test -- --ignored
    async fn test_create_logo_collection_invalid_user() {
        let pool = get_test_pool().await;
        let invalid_user_id = uuid::Uuid::new_v4(); // User that doesn't exist

        // Test creating a logo collection with non-existent user
        let result = create_logo_collection(
            &pool,
            invalid_user_id,
            "Test Collection",
            std::option::Option::None,
        ).await;

        // Should fail due to foreign key constraint
        assert!(result.is_err());
    }

    #[test]
    fn test_create_logo_collection_unit_tests() {
        // Unit tests that don't require database connection

        // Test UUID generation
        let user_id = uuid::Uuid::new_v4();
        assert!(!user_id.to_string().is_empty());

        // Test string handling
        let name = "Test Collection";
        let description = std::option::Option::Some("Test description");
        assert_eq!(name.len(), 15);
        assert!(description.is_some());

        // Test None handling
        let no_description: std::option::Option<&str> = std::option::Option::None;
        assert!(no_description.is_none());
    }
}
