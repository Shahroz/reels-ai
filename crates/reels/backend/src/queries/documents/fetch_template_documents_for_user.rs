#![allow(clippy::disallowed_methods)]
//! Fetches template documents for a specific user in Content Studio.
//!
//! This function executes a SQL query against the database to retrieve documents
//! marked as content studio templates via the "content_studio_template" source.
//! It allows for text-based searching within document titles and content, 
//! pagination via limit and offset parameters, and proper access control.
//! Adheres to Rust coding guidelines with FQN usage and one item per file.

pub async fn fetch_template_documents_for_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
    limit: i64,
    offset: i64,
) -> std::result::Result<std::vec::Vec<crate::db::documents::Document>, sqlx::Error> {
    let like_pattern = format!("%{search_pattern}%");
    
    let documents = sqlx::query_as::<_, crate::db::documents::Document>(
        r#"
        SELECT 
            id, user_id, title, content, sources, status, created_at, updated_at, 
            is_public, is_task, include_research, collection_id 
        FROM documents 
        WHERE (user_id = $1 OR is_public = true) 
        AND sources @> ARRAY['content_studio_template'] 
        AND (title ILIKE $2 OR content ILIKE $2) 
        ORDER BY updated_at DESC 
        LIMIT $3 OFFSET $4
        "#
    )
    .bind(user_id)
    .bind(like_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    std::result::Result::Ok(documents)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_fetch_template_documents_for_user() {
        // Test placeholder - would implement actual database test
        // Testing template document filtering and access control
        let user_id = uuid::Uuid::new_v4();
        let search_pattern = "";
        let limit = 10;
        let offset = 0;
        
        // Would test with actual database pool
        // assert!(fetch_template_documents_for_user(&pool, user_id, search_pattern, limit, offset).await.is_ok());
    }
}
