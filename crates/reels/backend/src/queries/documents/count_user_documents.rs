//! Counts documents for a user, including owned and shared documents.
//!
//! This function executes a SQL query to count distinct documents accessible to a user.
//! This includes documents they own, documents shared with them directly, and
//! documents shared with organizations they are a member of.
//! It supports filtering by a search pattern and task status.

pub async fn count_user_documents(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    org_ids: &[uuid::Uuid],
    search_pattern: &str,
    is_task_filter: Option<bool>,
) -> std::result::Result<i64, sqlx::Error> {
    let count_result = sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT d.id)
        FROM documents d
        LEFT JOIN object_shares os_user ON d.id = os_user.object_id
            AND os_user.object_type = 'document'
            AND os_user.entity_type = 'user'
            AND os_user.entity_id = $1
        LEFT JOIN object_shares os_org ON d.id = os_org.object_id
            AND os_org.object_type = 'document'
            AND os_org.entity_type = 'organization'
            AND os_org.entity_id = ANY($2)
        WHERE (d.user_id = $1 OR d.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL)
        AND (d.title ILIKE $3 OR d.content ILIKE $3)
        AND ($4::BOOLEAN IS NULL OR d.is_task = $4)
        "#,
        user_id,
        org_ids,
        search_pattern,
        is_task_filter
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(count_result.unwrap_or(0))
}