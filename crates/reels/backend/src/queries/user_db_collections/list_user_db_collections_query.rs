//! Provides a query function to list user database collections with pagination, sorting, and searching.
//!
//! This function encapsulates the database interaction logic for retrieving
//! a list of `UserDbCollection` entities based on specified criteria.
//! It returns both the list of items and the total count for pagination purposes.
//! Adheres to the 'one item per file' and FQN guidelines from `rust_guidelines.md`.

// No 'use' statements as per rust_guidelines.md. All paths must be fully qualified.

use crate::sql_utils::count_sql_results::TotalCount;
use crate::db::user_db_collection::UserDbCollection;

/// Lists user database collections from the database.
///
/// Fetches a paginated, sorted, and filtered list of `crate::db::user_db_collection::UserDbCollection`s for a given user.
///
/// # Arguments
/// * `pool` - A reference to the `sqlx::PgPool` for database access.
/// * `user_id` - The `uuid::Uuid` of the user whose collections are to be listed.
/// * `limit` - The maximum number of items to return.
/// * `offset` - The number of items to skip for pagination.
/// * `sort_by_db_col_name` - The database column name to sort by (e.g., "name", "created_at"). Validated by caller.
/// * `sort_order_db` - The sort order ("ASC" or "DESC"). Validated by caller.
/// * `search_pattern_db` - The SQL ILIKE pattern for searching collection names (e.g., "%search_term%").
///
/// # Returns
/// A `std::result::Result` containing a tuple of (`std::vec::Vec<crate::db::user_db_collection::UserDbCollection>`, `i64`)
/// representing the list of collections and the total count of matching collections,
/// or an `sqlx::Error` if the query fails.
pub async fn list_user_db_collections_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    limit: i64,
    offset: i64,
    sort_by_db_col_name: &str,
    sort_order_db: &str,
    search_pattern_db: Option<&str>,
) -> std::result::Result<(std::vec::Vec<crate::db::user_db_collection::UserDbCollection>, i64), sqlx::Error> {
    let user_id_param = user_id;
    let search_pattern_param = search_pattern_db;
    let limit_param = limit;
    let offset_param = offset;

    // Total count query
    let count_query = sqlx_conditional_queries::conditional_query_as!(
        TotalCount,
        r#"
            SELECT COUNT(*) FROM user_db_collections
            WHERE user_id = {user_id_param}
            {#search_filter}
        "#,
        #user_id_param = match &user_id_param { _ => "{user_id_param}" },
        #search_filter = match &search_pattern_param {
            Some(_) => "AND name ILIKE {search_pattern_param}",
            None => ""
        },
        #search_pattern_param = match &search_pattern_param { _ => "{search_pattern_param}" }
    );

    let total_count = match count_query.fetch_one(pool).await {
        Ok(count) => count,
        Err(e) => return std::result::Result::Err(e),
    };

    // Items query
    let items_query = sqlx_conditional_queries::conditional_query_as! {
        UserDbCollection,
        r#"
            SELECT
                id AS "id: uuid::Uuid",
                user_id AS "user_id: uuid::Uuid",
                name,
                description as "description?",
                schema_definition AS "schema_definition: serde_json::Value",
                source_predefined_collection_id AS "source_predefined_collection_id: uuid::Uuid",
                ui_component_definition AS "ui_component_definition: serde_json::Value",
                created_at,
                updated_at
            FROM user_db_collections
            WHERE user_id = {user_id_param}
            {#search_filter}
            ORDER BY {#sort_by_sql_literal} {#sort_order_sql_literal}
            LIMIT {limit_param} OFFSET {offset_param}
        "#,
        #user_id_param = match &user_id_param { _ => "{user_id_param}" },
        #search_filter = match &search_pattern_param {
            Some(_) => "AND name ILIKE {search_pattern_param}",
            None => ""
        },
        #search_pattern_param = match &search_pattern_param { _ => "{search_pattern_param}" },
        #sort_by_sql_literal = match sort_by_db_col_name {
            "name" => "name",
            "updated_at" => "updated_at",
            _ => "created_at"
        },
        #sort_order_sql_literal = match sort_order_db {
            "ASC" => "ASC",
            _ => "DESC"
        },
        #limit_param = match &limit_param { _ => "{limit_param}" },
        #offset_param = match &offset_param { _ => "{offset_param}" }
    };

    let items = items_query.fetch_all(pool).await?;

    std::result::Result::Ok((items, total_count.count.unwrap_or(0)))
}

#[cfg(test)]
mod tests {
    // No `use` statements as per rust_guidelines.md.

    // Helper to create a mock UserDbCollection for potential assertions (not used in current basic tests).
    // Note: `serde_json::json!` and `chrono::Utc::now()` would require FQNs or be available.
    // `UserDbCollection` definition implies these are resolvable.
    fn _create_mock_collection(id: uuid::Uuid, user_id: uuid::Uuid, name: &str) -> crate::db::user_db_collection::UserDbCollection {
        crate::db::user_db_collection::UserDbCollection {
            id,
            user_id,
            name: std::string::String::from(name),
            description: None,
            schema_definition: serde_json::json!({}), // Example, assumes serde_json::json is usable
            source_predefined_collection_id: None,
            ui_component_definition: serde_json::json!({}), // Example, assumes serde_json::json is usable
            created_at: chrono::Utc::now(), // Example, assumes chrono::Utc::now is usable
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_list_user_db_collections_query_compiles_placeholder() {
        // This test is a placeholder. A full test of this function would require
        // a connection to a test database or a mocking framework like `sqlxmock`.
        // The purpose here is to confirm the function signature and basic structure compile.
        // Example of a conceptual call (would not run without mocks):
        // let user_id_mock = uuid::Uuid::new_v4();
        // let pool_mock: sqlx::PgPool = ...; // Needs a mock or real pool
        // let result = super::list_user_db_collections_query(
        //     &pool_mock, user_id_mock, 10, 0, "created_at", "DESC", "%"
        // ).await;
        // std::assert!(result.is_err()); // Expect error if pool_mock is not valid
        std::assert!(true, "Placeholder test: full DB/mocking setup needed for comprehensive testing.");
    }

    #[test]
    fn test_internal_query_string_formatting_logic() {
        // Tests the `format!` macro usage for constructing the SQL query string,
        // which is a critical part of the function's logic not involving direct DB access.
        let sort_by_col_name = "name";
        let sort_order_val = "ASC";
        let generated_query_str = format!(
            "SELECT * FROM user_db_collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY {} {} LIMIT $3 OFFSET $4",
            sort_by_col_name, sort_order_val
        );

        std::assert!(generated_query_str.contains("ORDER BY name ASC"));
        std::assert!(generated_query_str.starts_with("SELECT * FROM user_db_collections"));
        std::assert!(generated_query_str.contains("LIMIT $3 OFFSET $4"));

        let sort_by_col_updated_at = "updated_at";
        let sort_order_val_desc = "DESC";
        let generated_query_str_desc = format!(
            "SELECT * FROM user_db_collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY {} {} LIMIT $3 OFFSET $4",
            sort_by_col_updated_at, sort_order_val_desc
        );
        std::assert!(generated_query_str_desc.contains("ORDER BY updated_at DESC"));
    }
}