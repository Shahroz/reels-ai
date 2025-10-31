//! Defines the query for listing collections for a user with pagination and sorting.
//!
//! This function retrieves a paginated list of collections for a given user,
//! with support for searching by name, and sorting by name, creation date, or update date.
//! Adheres to the one-item-per-file and FQN guidelines.

pub async fn list_collections(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<std::vec::Vec<crate::db::collections::Collection>, sqlx::Error> {
    // This dynamic query is kept as a match expression for clarity and to prevent SQL injection.
    // An alternative would be a query builder.
    match (sort_by, sort_order) {
        ("name", "asc") => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY name asc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
        ("name", "desc") => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY name desc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
        ("created_at", "asc") => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY created_at asc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
        ("created_at", "desc") => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY created_at desc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
        ("updated_at", "asc") => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY updated_at asc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
        ("updated_at", "desc") => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY updated_at desc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
        _ => sqlx::query_as!(crate::db::collections::Collection, "SELECT * FROM collections WHERE user_id = $1 AND name ILIKE $2 ORDER BY created_at desc LIMIT $3 OFFSET $4", user_id, search_pattern, limit, offset).fetch_all(pool).await,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_list_collections_query_placeholder() {
        // To be implemented with a test database.
        assert!(true);
    }
}