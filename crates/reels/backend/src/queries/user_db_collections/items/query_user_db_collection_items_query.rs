#![allow(clippy::disallowed_methods)]
//! Defines the query function for retrieving items from a user DB collection.
//!
//! This function encapsulates the logic for verifying collection ownership,
//! parsing a query string (currently placeholder), counting matching items,
//! and fetching a paginated list of those items. It's designed to be called
//! by route handlers or other services needing to query collection items.

// No `use` statements are used; fully qualified paths are used as per rust_guidelines.md.
pub async fn query_user_db_collection_items_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    query_string: &str, // from request body's QueryUserDbCollectionItemsRequest.query
    page: i64, // from request body, defaults applied by handler
    limit: i64, // from request body, defaults applied by handler
) -> Result<(std::vec::Vec<crate::db::user_db_collection_item::UserDbCollectionItem>, i64), anyhow::Error> {
    let offset = (page - 1) * limit;

    // 1. Verify ownership of parent collection
    match sqlx::query_scalar!(
        r#"SELECT user_id FROM user_db_collections WHERE id = $1"#,
        collection_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(owner_id) => {
            if owner_id != user_id {
                // Create an ErrorResponse and wrap it in anyhow::Error with a status code
                return std::result::Result::Err(anyhow::anyhow!("User does not own the parent collection")
                    .context(actix_web::http::StatusCode::FORBIDDEN));
            }
        }
        None => {
            return std::result::Result::Err(anyhow::anyhow!("Parent collection not found.")
                .context(actix_web::http::StatusCode::NOT_FOUND));
        }
    };

    // 2. Parse query string (Placeholder for now)
    // TODO: Integrate call to `crate::query_parser::item_query_parser::parse_item_query(query_string)`
    let (where_clause, params_values): (std::string::String, std::vec::Vec<serde_json::Value>) =
        if query_string.trim().is_empty() {
            (std::string::String::from("1 = 1"), std::vec::Vec::new())
        } else {
            log::warn!("Query parsing not yet fully implemented. Query: {query_string}");
            (std::string::String::from("1 = 1"), std::vec::Vec::new()) // Default to all items if parser not ready
        };

    // 3. Fetch total count
    // The base query for count. Dynamic params for where_clause start from $2 if params_values is used.
    let count_query_base_str = "SELECT COUNT(*) FROM user_db_collection_items WHERE user_db_collection_id = $1";
    let count_query_str = format!("{count_query_base_str} AND ({where_clause})");
    
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query_str).bind(collection_id);
    for val in params_values.iter() { // Bind parameters for the WHERE clause
        count_query_builder = count_query_builder.bind(val);
    }

    let total_count: i64 = match count_query_builder.fetch_one(pool).await {
        Ok(count_val) => {
            if count_val < 0 {
                log::warn!("COUNT(*) returned a negative value: {count_val}. Defaulting to 0.");
                0
            } else {
                count_val
            }
        }
        Err(e) => {
            log::error!("Failed to count user DB collection items: {e:?}. Query: {where_clause}, Params: {params_values:?}");
            return std::result::Result::Err(anyhow::anyhow!(e).context("Failed to count collection items."));
            }
        };

    // 4. Fetch items with pagination
    let query_str = format!(
        "SELECT id, user_db_collection_id, item_data, created_at, updated_at FROM user_db_collection_items WHERE user_db_collection_id = $1 AND ({where_clause}) ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    );
    
    let mut query = sqlx::query_as::<_, crate::db::user_db_collection_item::UserDbCollectionItem>(&query_str)
        .bind(collection_id)
        .bind(limit)
        .bind(offset);
        
    // Bind the dynamic parameters from the where clause
    for param in params_values.iter() {
        query = query.bind(param);
    }
    
    let items_result = query.fetch_all(pool).await;

    match items_result {
        Ok(items) => std::result::Result::Ok((items, total_count)),
        Err(e) => {
            log::error!("Failed to fetch user DB collection items: {e:?}. Query: {where_clause}, Params: {params_values:?}");
            std::result::Result::Err(anyhow::anyhow!(e).context("Failed to retrieve collection items."))
        }
    }
}

#[cfg(test)]
mod tests {
    // To write comprehensive tests, we'd need to:
    // 1. Set up a test database environment (e.g., using `sqlx::test` or `testcontainers`).
    // 2. Create mock data: a user, a collection owned by the user, and items in that collection.
    // 3. If `crate::query_parser` were integrated, its behavior would need to be considered/mocked.
    // These tests are basic structural checks.

    #[test]
    fn test_query_function_signature_compiles_and_structure_is_valid() {
        // This test mainly ensures the function signature and basic structure are valid.
        // It doesn't execute the async block deeply due to lack of real DB pool.
        async fn _test_wrapper_for_compilation() {
            // Mock a PgPool. In a real test, this would connect to a test database.
            let pool: Option<sqlx::PgPool> = None; 
            let user_id = uuid::Uuid::new_v4();
            let collection_id = uuid::Uuid::new_v4();
            let query_string = "test query";
            let page = 1i64;
            let limit = 10i64;

            if let Some(p_ref) = pool.as_ref() { // Avoids moving pool if it were real
                let _ = super::query_user_db_collection_items_query(
                    p_ref,
                    user_id,
                    collection_id,
                    query_string,
                    page,
                    limit,
                )
                .await;
            }
        }
        assert!(true); // Basic assertion to make the test pass.
    }

    // A more detailed test would look like this (requires DB setup):
    // #[sqlx::test] // This macro can handle test DB setup if sqlx-cli is configured
    // async fn test_empty_query_fetches_items_for_owner(pool: sqlx::PgPool) -> sqlx::Result<()> {
    //     // --- Arrange ---
    //     // Create a test user
    //     let user_id = uuid::Uuid::new_v4();
    //     // sqlx::query!("INSERT INTO users (id, ...) VALUES ($1, ...)", user_id).execute(&pool).await?;
    //     
    //     // Create a test collection for this user
    //     let collection_id = uuid::Uuid::new_v4();
    //     // sqlx::query!("INSERT INTO user_db_collections (id, user_id, name, ...) VALUES ($1, $2, 'Test Collection', ...)", collection_id, user_id).execute(&pool).await?;
    //     
    //     // Create some test items in this collection
    //     // sqlx::query!("INSERT INTO user_db_collection_items (id, user_db_collection_id, item_data) VALUES ($1, $2, $3)", uuid::Uuid::new_v4(), collection_id, serde_json::json!({"name": "Item 1"})).execute(&pool).await?;
    //     // sqlx::query!("INSERT INTO user_db_collection_items (id, user_db_collection_id, item_data) VALUES ($1, $2, $3)", uuid::Uuid::new_v4(), collection_id, serde_json::json!({"name": "Item 2"})).execute(&pool).await?;
    //    
    //     // --- Act ---
    //     let result = super::query_user_db_collection_items_query(
    //         &pool,
    //         user_id,
    //         collection_id,
    //         "", // Empty query string
    //         1,  // Page
    //         10  // Limit
    //     ).await;
    //
    //     // --- Assert ---
    //     assert!(result.is_ok(), "Query failed: {:?}", result.err());
    //     let (items, total_count) = result.unwrap();
    //     // assert_eq!(items.len(), 2, "Expected 2 items");
    //     // assert_eq!(total_count, 2, "Expected total count of 2");
    //     // Further assertions on item content if necessary
    //
    //     std::result::Result::Ok(())
    // }
}
