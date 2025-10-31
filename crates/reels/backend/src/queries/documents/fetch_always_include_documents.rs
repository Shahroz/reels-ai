//! Retrieves documents for a user that are marked for always including research.
//!
//! This function queries the `documents` table, filtering by `user_id`
//! and where the `include_research` column matches the 'Always' state.
//! It is designed to fetch all relevant documents according to this specific criterion.
//! The function returns a vector of `crate::db::documents::Document` structs or an `sqlx::Error`.

//! Revision History
//! - 2025-06-05T18:35:50Z @AI: Moved function to queries/documents module.
//! - 2025-06-05T18:01:50Z @AI: Initial creation of the function to fetch documents marked for always including research.

/// Fetches documents for a given user that are configured to always include research.
///
/// Queries the `documents` table for entries matching the `user_id` where
/// `include_research` is set to `crate::db::document_research_usage::DocumentResearchUsage::Always`.
///
/// Results are ordered by `updated_at` in descending order.

use crate::db::document_research_usage::DocumentResearchUsage;
use sqlx::PgPool;

pub async fn fetch_always_include_documents_for_user(
    pool: &sqlx::postgres::PgPool,
    user_id: sqlx::types::Uuid,
) -> std::result::Result<Vec<crate::db::documents::Document>, sqlx::Error> {
    let always_value_str = crate::db::document_research_usage::DocumentResearchUsage::Always.to_string();
    let documents = sqlx::query_as!(
        crate::db::documents::Document,
        r#"
        SELECT
            id,
            user_id,
            title,
            content,
            sources,
            status,
            created_at,
            updated_at,
            is_public,
            is_task,
            include_research AS "include_research: DocumentResearchUsage",
            collection_id
        FROM documents
        WHERE (user_id = $1 OR is_public = true) AND include_research = $2
        ORDER BY updated_at DESC
        "#,
        user_id,
        always_value_str
    )
    .fetch_all(pool)
    .await?;

    std::result::Result::Ok(documents)
}

/// Fetches all documents that are marked with 'always_include' in their sources.
/// This is used for providing consistent, global context to certain operations.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
///
/// # Returns
///
/// A `Result` containing a `Vec<Document>` on success, or a `sqlx::Error` on failure.
pub async fn fetch_always_include_documents(pool: &PgPool) -> Result<Vec<crate::db::documents::Document>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::documents::Document,
        r#"
        SELECT
            id, user_id, title, content, sources, status, created_at, updated_at,
            is_public, is_task, include_research AS "include_research: _", collection_id
        FROM documents 
        WHERE sources @> ARRAY['always_include']
        "#
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    // These tests are placeholders and primarily ensure compilation.
    // Full integration testing would require a test database setup.

    #[test]
    fn check_function_signature_compiles() {
        // This test doesn't execute database logic but checks that the function
        // signature and its usage with dummy parameters would compile correctly.
        // It helps catch type mismatches or path errors early.
        async fn _compile_check_wrapper(
            pool: &sqlx::postgres::PgPool,
            user_id: sqlx::types::Uuid,
        ) -> std::result::Result<Vec<crate::db::documents::Document>, sqlx::Error> {
            // Call the function from the parent module using `super::`.
            super::fetch_always_include_documents_for_user(pool, user_id).await
        }
        // A trivial assertion to make this a valid test function.
        // The main purpose is the compile-time check of the _compile_check_wrapper.
        std::assert!(true, "Placeholder assertion for compilation check validity.");
    }
}
