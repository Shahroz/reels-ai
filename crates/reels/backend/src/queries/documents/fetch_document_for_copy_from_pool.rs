//! Fetches a document's details along with the requesting user's access level using pool connections.
//!
//! This query is specifically designed for the copy operation. It retrieves the
//! document's content and metadata, and crucially, determines if the user has
//! rights to view it (either as owner, through a direct or organizational share,
//! or because the document is public). This pool-based version is used for
//! read-only operations before starting transactions, optimizing connection usage.

/// Holds the necessary information about the original document for the copy operation.
#[derive(sqlx::FromRow, std::fmt::Debug)]
pub struct OriginalDocumentAccessInfo {
    pub id: uuid::Uuid,
    pub user_id: std::option::Option<uuid::Uuid>, // Original owner
    pub title: std::string::String,
    pub content: std::string::String,
    pub sources: std::vec::Vec<std::string::String>,
    pub status: std::string::String,
    pub is_public: bool,
    pub is_task: bool,
    pub include_research: std::option::Option<std::string::String>,
    pub current_user_access_to_original: std::option::Option<std::string::String>,
}

/// Fetches document details and access permissions using pool connections.
///
/// This function retrieves the document's content and metadata, and determines
/// if the user has rights to view it. It uses pool connections for read-only
/// operations before any transactions are started, optimizing connection usage
/// and performance for the copy operation.
pub async fn fetch_document_for_copy_from_pool(
    pool: &sqlx::PgPool,
    document_id: uuid::Uuid,
    user_id: uuid::Uuid,
    organization_ids: &[uuid::Uuid],
) -> std::result::Result<std::option::Option<OriginalDocumentAccessInfo>, sqlx::Error> {
    sqlx::query_as!(
        OriginalDocumentAccessInfo,
        r#"
        WITH UserOrgShares AS (
            SELECT access_level
            FROM object_shares
            WHERE object_id = $1 AND entity_id = $2 AND entity_type = 'user'::object_share_entity_type AND object_type = 'document'
        UNION ALL
            SELECT os.access_level
            FROM object_shares os
            WHERE os.object_id = $1 AND os.entity_id = ANY($3::UUID[]) AND os.entity_type = 'organization'::object_share_entity_type AND os.object_type = 'document'
        ),
        RankedUserOrgShares AS (
            SELECT access_level, ROW_NUMBER() OVER (ORDER BY CASE access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END) as rn
            FROM UserOrgShares
        )
        SELECT
            d.id, d.user_id, d.title, d.content, d.sources, d.status, d.is_public,
            d.is_task, d.include_research,
            COALESCE(
                (SELECT rus.access_level::TEXT FROM RankedUserOrgShares rus WHERE rus.rn = 1),
                CASE WHEN d.user_id = $2 THEN 'owner' ELSE NULL END
            ) AS current_user_access_to_original
        FROM documents d
        WHERE d.id = $1
        "#,
        document_id,    // $1
        user_id,        // $2
        organization_ids // $3: &[Uuid] passed for ANY($N::UUID[])
    )
    .fetch_optional(pool)
    .await
}

// Note: This function contains complex SQL logic that requires a database connection.
// Document copy permissions are thoroughly tested in the integration test suite. 