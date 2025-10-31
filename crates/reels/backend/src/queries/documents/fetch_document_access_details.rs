//! Defines the query for fetching detailed document access information for permission checks.
//!
//! This query gathers the document's owner, its public status, and any direct or
//! organization-based shares for a specific user. This is used to determine if a user
//! has the right to view or edit a document.

/// Holds detailed information about a document and the access level of the current user.
#[derive(sqlx::FromRow, Debug)]
pub struct DocumentAccessDetails {
    pub owner_user_id: Option<uuid::Uuid>,
    pub creator_email: Option<String>,
    pub shared_access_level: Option<String>,
    pub is_public: bool,
    pub is_task: bool,
    pub include_research: Option<String>,
    pub id: uuid::Uuid,
    pub title: String,
    pub content: String,
    pub sources: std::vec::Vec<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Fetches comprehensive access details for a single document.
pub async fn fetch_document_access_details(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: uuid::Uuid,
    authenticated_user_id: uuid::Uuid,
    org_ids: &[uuid::Uuid],
) -> std::result::Result<Option<DocumentAccessDetails>, sqlx::Error> {
    sqlx::query_as!(
        DocumentAccessDetails,
        r#"
        WITH UserOrgShares AS (
            SELECT access_level
            FROM object_shares
            WHERE object_id = $1 AND entity_id = $2 AND entity_type = 'user'::object_share_entity_type AND object_type = 'document'
        UNION ALL
            SELECT os.access_level
            FROM object_shares os
            WHERE os.object_id = $1 AND os.entity_id = ANY($3::UUID[]) AND os.entity_type = 'organization'::object_share_entity_type AND object_type = 'document'
        ),
        RankedUserOrgShares AS (
            SELECT access_level, ROW_NUMBER() OVER (ORDER BY CASE access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END) as rn
            FROM UserOrgShares
        )
        SELECT
            d.user_id AS owner_user_id,
            (SELECT email FROM users WHERE id = d.user_id) AS creator_email,
            (SELECT rus.access_level::TEXT FROM RankedUserOrgShares rus WHERE rus.rn = 1) AS shared_access_level,
            d.is_public,
            d.is_task,
            d.include_research::TEXT,
            d.id,
            d.title,
            d.content,
            d.sources,
            d.status,
            d.created_at,
            d.updated_at
        FROM documents d
        WHERE d.id = $1
        "#,
        document_id,
       authenticated_user_id,
       org_ids
   )
   .fetch_optional(&mut **tx)
   .await
}
