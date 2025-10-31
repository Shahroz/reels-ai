#![allow(clippy::disallowed_methods)]
//! Fetches a paginated and sorted list of documents for a specific user.
//!
//! This function executes a SQL query against the database to retrieve documents
//! matching the given user ID. It allows for text-based searching within document
//! titles and content. Pagination is supported via limit and offset parameters,
//! and results can be sorted by a specified column and order.
//! Adheres to FQN and no-`use` statements guidelines.

// Fully qualified paths are used throughout this file as per guidelines.
// Removed import of Document; using fully qualified path in macro invocation.

pub async fn fetch_documents_for_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
    limit: i64,
    offset: i64,
   sort_by: &str,
   sort_order: &str,
) -> std::result::Result<std::vec::Vec<crate::db::documents::Document>, sqlx::Error> {
    // Build safe dynamic SQL for pagination, search, and sorting.
    let sort_by_sql = match sort_by.to_lowercase().as_str() {
        "title" => "title",
        "status" => "status",
        "created_at" => "created_at",
        "updated_at" => "updated_at",
        _ => "created_at",
    };

    let sort_order_sql = match sort_order.to_lowercase().as_str() {
        "asc" => "ASC",
        "desc" => "DESC",
        _ => "DESC",
    };

    let like_pattern = format!("%{search_pattern}%");

    let documents = sqlx::query_as::<_, crate::db::documents::Document>(
        &format!(
            "SELECT id, user_id, title, content, sources, status, created_at, updated_at, is_public, is_task, include_research, collection_id FROM documents \
WHERE (user_id = $1 OR is_public = true) AND (title ILIKE $2 OR content ILIKE $2) \
ORDER BY {sort_by_sql} {sort_order_sql} \
LIMIT $3 OFFSET $4"
        ),
    )
    .bind(user_id)
    .bind(like_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(documents)
}

// As per rust_guidelines, unit tests for this function would typically be included below.
// #[cfg(test)]
// mod tests {
//     // Example test structure:
//     // #[test]
//     // fn test_fetch_basic_scenario() {
//     //     // 1. Setup: Initialize a test database or mock pool.
//     //     //    Populate with test data if necessary.
//     //     //    Define user_id, search_pattern, limit, offset, sort_by, sort_order.
//     //
//     //     // 2. Act: Call super::fetch_documents_for_user(...).
//     //     //    let result = super::fetch_documents_for_user(&pool, user_id, ...).await;
//     //
//     //     // 3. Assert: Check if the result is Ok and contains the expected documents.
//     //     //    Verify pagination, sorting, and filtering logic.
//     //     //    std::assert!(result.is_ok());
//     //     //    let documents = result.unwrap();
//     //     //    std::assert_eq!(documents.len(), expected_number_of_documents);
//     //     //    // Further assertions on document content...
//     // }
//     //
//     // #[test]
//     // fn test_fetch_empty_result() {
//     //     // Test scenario where no documents match the criteria.
//     // }
//     //
//     // #[test]
//     // fn test_fetch_with_error() {
//     //     // Test scenario that might cause a database error (e.g., invalid sort_by column if not handled by query builder).
//     // }
// }
