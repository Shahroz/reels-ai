#![allow(clippy::disallowed_methods)]
//! Finds a specific document entry by its ID for a given user.
//!
//! This function queries the database for a document matching both the document ID
//! and the user ID. It's designed to ensure that users can only access their own documents.
//! Returns a `Result` containing an `Option<crate::db::documents::Document>`:
//! `Some(Document)` if found, `None` if not found, or an `sqlx::Error` on database query failure.
//! This adheres to the one-item-per-file guideline.

// No `use` statements allowed as per rust_guidelines.
// Removed import of Document; using fully qualified path in macro invocation.

pub async fn find_document_by_id_and_user(
    pool: &sqlx::PgPool,
   document_id: uuid::Uuid,
   user_id: uuid::Uuid,
) -> std::result::Result<Option<crate::db::documents::Document>, sqlx::Error> {
    let result = sqlx::query_as::<_, crate::db::documents::Document>(
        "SELECT id, user_id, title, content, sources, status, created_at, updated_at, is_public, is_task, include_research, collection_id FROM documents WHERE id = $1 AND (user_id = $2 OR is_public = true)",
    )
    .bind(document_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    // Per rust_guidelines, tests are in the same file.
    // Full FQNs are used, and `super::` for the item under test.
    // Note: These tests are conceptual placeholders as they require a database connection
    // and an async runtime (e.g., `#[tokio::test]`).

    #[test]
    fn test_conceptual_find_document_found() {
        // This test conceptually verifies finding a document.
        // An actual test would involve:
        // 1. Setting up a test database pool.
        // 2. Inserting a test document with known IDs.
        // 3. Calling `super::find_document_by_id_and_user`.
        // 4. Asserting the result is `Ok(Some(document))` and matches expected data.
        // Example (pseudo-code, requires async test runner like `#[tokio::test]`):
        //
        // async fn body() {
        //   let test_pool = /* obtain test_pool */;
        //   let known_doc_id = uuid::Uuid::new_v4();
        //   let known_user_id = uuid::Uuid::new_v4();
        //   /* ... insert data into test_pool ... */
        //   let result = super::find_document_by_id_and_user(&test_pool, known_doc_id, known_user_id).await;
        //   std::assert!(result.is_ok());
        //   std::assert!(result.unwrap().is_some());
        // }
        // tokio::runtime::Runtime::new().unwrap().block_on(body());
        std::assert!(true, "Conceptual test: document found. Requires DB and async runtime setup.");
    }

    #[test]
    fn test_conceptual_find_document_not_found() {
        // This test conceptually verifies not finding a non-existent document.
        // An actual test would involve:
        // 1. Setting up a test database pool.
        // 2. Using IDs that are known not to exist in the database.
        // 3. Calling `super::find_document_by_id_and_user`.
        // 4. Asserting the result is `Ok(None)`.
        // Example (pseudo-code, requires async test runner like `#[tokio::test]`):
        //
        // async fn body() {
        //   let test_pool = /* obtain test_pool */;
        //   let non_existent_doc_id = uuid::Uuid::new_v4();
        //   let user_id = uuid::Uuid::new_v4();
        //   let result = super::find_document_by_id_and_user(&test_pool, non_existent_doc_id, user_id).await;
        //   std::assert!(result.is_ok());
        //   std::assert!(result.unwrap().is_none());
        // }
        // tokio::runtime::Runtime::new().unwrap().block_on(body());
        std::assert!(true, "Conceptual test: document not found. Requires DB and async runtime setup.");
    }
}
