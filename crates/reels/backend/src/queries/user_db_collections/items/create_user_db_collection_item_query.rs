//! Defines the query function for creating a user database collection item.
//!
//! This function encapsulates the logic for verifying collection ownership,
//! validating the item data against the collection's schema, and inserting
//! the new item into the database. It adheres to the 'one item per file'
//! and FQN guidelines from `rust_guidelines.md`.

// Note: No `use` statements as per guidelines. Fully qualified paths are used.

/// Custom error types for the `create_user_db_collection_item_query` function.
#[derive(Debug)]
pub enum QueryError {
    Forbidden(std::string::String),
    NotFound(std::string::String),
    Validation(std::string::String),
    SchemaCompilation(std::string::String),
    Database(std::string::String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            QueryError::NotFound(msg) => write!(f, "Not Found: {msg}"),
            QueryError::Validation(msg) => write!(f, "Validation Error: {msg}"),
            QueryError::SchemaCompilation(msg) => write!(f, "Schema Compilation Error: {msg}"),
            QueryError::Database(msg) => write!(f, "Database Error: {msg}"),
        }
    }
}

impl std::error::Error for QueryError {}


/// Creates a new item in a user DB collection.
///
/// Verifies ownership, validates data against schema, and inserts the item.
/// Returns the created item or an error detailing what went wrong.
pub async fn create_user_db_collection_item_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_uuid: uuid::Uuid,
    item_data: serde_json::Value,
) -> std::result::Result<crate::db::user_db_collection_item::UserDbCollectionItem, anyhow::Error> {
    // 1. Verify ownership of parent collection and get schema_definition
    let collection_details_opt = match sqlx::query!(
        r#"
        SELECT user_id, schema_definition
        FROM user_db_collections
        WHERE id = $1
        "#,
        collection_id_uuid
    )
    .fetch_optional(pool)
    .await
    {
        Ok(record_opt) => record_opt,
        Err(e) => {
            log::error!("Failed to fetch parent collection for item creation (query): {e:?}");
            return std::result::Result::Err(anyhow::Error::new(QueryError::Database(
                std::string::String::from("Failed to verify collection ownership."),
            )).context(e));
        }
    };

    let collection_schema_definition = match collection_details_opt {
        Some(record) => {
            if record.user_id != user_id {
                return std::result::Result::Err(anyhow::Error::new(QueryError::Forbidden(
                    std::string::String::from("User does not own the parent collection."),
                )));
            }
            record.schema_definition
        }
        None => {
            return std::result::Result::Err(anyhow::Error::new(QueryError::NotFound(
                std::string::String::from("Parent collection not found."),
            )));
       }
    };

  // 2. Validate item_data against schema_definition
   let compiled_schema = match jsonschema::Validator::new(&collection_schema_definition) {
      Ok(s) => s,
      Err(e) => {
           // Create an owned string from the error to ensure 'static lifetime for anyhow.
            // Include collection_id_uuid in the error_message for better context.
            let error_message = std::format!(
                "Invalid schema definition for collection {collection_id_uuid}. Schema compilation error: {e}"
            );
            log::error!(
                // Log the detailed message and the original error structure for debugging.
                "Schema compilation failed for collection_id {collection_id_uuid}: {error_message}. Original jsonschema error: {e:?}" // Log the original error structure for detailed debugging if needed
            );
            return std::result::Result::Err(anyhow::Error::new(QueryError::SchemaCompilation(
                error_message, // Pass the owned, 'static error message.
            )));
        }
    };

    if let Err(validation_errors) = compiled_schema.validate(&item_data) {
        let error_messages: std::vec::Vec<std::string::String> = vec![validation_errors.to_string()];
        let error_string = error_messages.join(", ");
        return std::result::Result::Err(anyhow::Error::new(QueryError::Validation(std::format!(
            "Item data does not conform to schema: {error_string}"
        ))));
    }

    // 3. Insert new item
    let new_item_id = uuid::Uuid::new_v4();
    match sqlx::query_as!(
        crate::db::user_db_collection_item::UserDbCollectionItem,
        r#"
        INSERT INTO user_db_collection_items (id, user_db_collection_id, item_data)
        VALUES ($1, $2, $3)
        RETURNING id, user_db_collection_id, item_data, created_at, updated_at
        "#,
        new_item_id,
        collection_id_uuid,
        item_data,
    )
    .fetch_one(pool)
    .await
    {
        Ok(item) => std::result::Result::Ok(item),
        Err(e) => {
            log::error!("Failed to create user DB collection item (query): {e:?}");
            std::result::Result::Err(anyhow::Error::new(QueryError::Database(
                std::string::String::from("Failed to create collection item."),
            )).context(e))
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Per guidelines, tests are in the same file.
    // Access to `super::create_user_db_collection_item_query` and `super::QueryError`.
    // These tests require a running PostgreSQL test database accessible via TEST_DATABASE_URL.
    // Tables `user_db_collections` and `user_db_collection_items` must exist.

    // Helper function to get a test pool. Panics if TEST_DATABASE_URL is not set or connection fails.
    async fn get_test_pool() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for integration tests.");
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1) // Keep connections low for tests
            .connect(&database_url)
            .await
            .expect("Failed to create test pool.")
    }

    // Helper to ensure a collection exists for testing.
    async fn ensure_test_collection(
        pool: &sqlx::PgPool,
        collection_id: uuid::Uuid,
        user_id: uuid::Uuid,
        schema: serde_json::Value,
    ) {
        // Upsert to handle cases where tests might run multiple times or in parallel with same IDs
        sqlx::query!(
            r#"
            INSERT INTO user_db_collections (id, user_id, name, description, schema_definition, created_at, updated_at) 
            VALUES ($1, $2, 'Test Collection', 'A test collection for query tests', $3, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET 
                user_id = EXCLUDED.user_id, 
                schema_definition = EXCLUDED.schema_definition, 
                updated_at = NOW()
            "#,
            collection_id,
            user_id,
            schema
        )
        .execute(pool)
        .await
        .expect("Failed to ensure test collection.");
    }

    // Helper to clean up a created item.
    async fn cleanup_test_item(pool: &sqlx::PgPool, item_id: uuid::Uuid) {
        sqlx::query!("DELETE FROM user_db_collection_items WHERE id = $1", item_id)
            .execute(pool)
            .await
            .ok(); // Best effort cleanup
    }
    
    // Helper to clean up a test collection.
    async fn cleanup_test_collection(pool: &sqlx::PgPool, collection_id: uuid::Uuid) {
        sqlx::query!("DELETE FROM user_db_collections WHERE id = $1", collection_id)
            .execute(pool)
            .await
            .ok(); // Best effort cleanup
    }

    #[tokio::test]
    async fn test_create_item_success() {
        if std::env::var("TEST_DATABASE_URL").is_err() {
            println!("Skipping DB test test_create_item_success: TEST_DATABASE_URL not set.");
            return;
        }
        let pool = get_test_pool().await;
        let user_id = uuid::Uuid::new_v4();
        let collection_id = uuid::Uuid::new_v4();
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"]
        });
        ensure_test_collection(&pool, collection_id, user_id, schema.clone()).await;

        let item_data = serde_json::json!({"name": "Test Item Alpha"});
        let result = super::create_user_db_collection_item_query(
            &pool,
            user_id,
            collection_id,
            item_data.clone(),
        )
        .await;

        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());
        if let std::result::Result::Ok(item) = result {
            assert_eq!(item.user_db_collection_id, collection_id);
            assert_eq!(item.item_data, item_data);
            cleanup_test_item(&pool, item.id).await;
        }
        cleanup_test_collection(&pool, collection_id).await;
    }

    #[tokio::test]
    async fn test_create_item_forbidden_wrong_user() {
        if std::env::var("TEST_DATABASE_URL").is_err() {
            println!("Skipping DB test test_create_item_forbidden_wrong_user: TEST_DATABASE_URL not set.");
            return;
        }
        let pool = get_test_pool().await;
        let owner_user_id = uuid::Uuid::new_v4();
        let other_user_id = uuid::Uuid::new_v4(); // Different user
        let collection_id = uuid::Uuid::new_v4();
        let schema = serde_json::json!({"type": "object"});
        ensure_test_collection(&pool, collection_id, owner_user_id, schema.clone()).await;

        let item_data = serde_json::json!({"data": "some data"});
        let result = super::create_user_db_collection_item_query(
            &pool,
            other_user_id, // This user does not own the collection
            collection_id,
            item_data,
        )
        .await;

        assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            assert!(matches!(e.downcast_ref::<super::QueryError>(), Some(super::QueryError::Forbidden(_))), "Expected Forbidden error, got {:?}", e);
        }
        cleanup_test_collection(&pool, collection_id).await;
    }

    #[tokio::test]
    async fn test_create_item_collection_not_found() {
         if std::env::var("TEST_DATABASE_URL").is_err() {
            println!("Skipping DB test test_create_item_collection_not_found: TEST_DATABASE_URL not set.");
            return;
        }
        let pool = get_test_pool().await;
        let user_id = uuid::Uuid::new_v4();
        let non_existent_collection_id = uuid::Uuid::new_v4(); // This ID won't be in the DB
        let item_data = serde_json::json!({"data": "any data"});

        let result = super::create_user_db_collection_item_query(
            &pool,
            user_id,
            non_existent_collection_id,
            item_data,
        )
        .await;

        assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
             assert!(matches!(e.downcast_ref::<super::QueryError>(), Some(super::QueryError::NotFound(_))), "Expected NotFound error, got {:?}", e);
        }
    }
    
    #[tokio::test]
    async fn test_create_item_schema_validation_fails() {
        if std::env::var("TEST_DATABASE_URL").is_err() {
            println!("Skipping DB test test_create_item_schema_validation_fails: TEST_DATABASE_URL not set.");
            return;
        }
        let pool = get_test_pool().await;
        let user_id = uuid::Uuid::new_v4();
        let collection_id = uuid::Uuid::new_v4();
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"] // 'name' is required
        });
        ensure_test_collection(&pool, collection_id, user_id, schema.clone()).await;

        let item_data = serde_json::json!({"age": 30}); // Data is missing the required 'name' field
        let result = super::create_user_db_collection_item_query(
            &pool,
            user_id,
            collection_id,
            item_data,
        )
        .await;

        assert!(result.is_err());
        if let std::result::Result::Err(e) = result {
            assert!(matches!(e.downcast_ref::<super::QueryError>(), Some(super::QueryError::Validation(_))), "Expected Validation error, got {:?}", e);
        }
        cleanup_test_collection(&pool, collection_id).await;
    }
}
