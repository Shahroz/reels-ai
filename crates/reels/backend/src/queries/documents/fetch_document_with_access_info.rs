//! Fetches a document and its associated access control information.
//!
//! This function queries for a single document by its ID, while also calculating
//! the access level for the currently authenticated user based on direct ownership,
//! direct shares, and organization-based shares.

use uuid::Uuid;

/// Represents a document record enriched with access level information for a specific user.
#[derive(sqlx::FromRow, Debug)]
pub struct DocumentWithAccessInfo {
    pub id: Uuid,
    pub user_id: Option<Uuid>, // Document's original owner/creator
    pub title: String,
    pub content: String,
    pub sources: Vec<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_public: bool,
    pub is_task: bool,
    pub include_research: Option<String>,
    pub creator_email: Option<String>, // Email of the document's original owner/creator
    pub shared_access_level: Option<String>, // Access level from object_shares ('viewer', 'editor')
}

pub async fn fetch_document_with_access_info(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document_id: Uuid,
    authenticated_user_id: Uuid,
    organization_ids: &[Uuid],
) -> std::result::Result<Option<DocumentWithAccessInfo>, sqlx::Error> {
    sqlx::query_as!(
        DocumentWithAccessInfo,
        r#"
        WITH UserOrgShares AS (
            -- Direct user shares for this document
            SELECT access_level
            FROM object_shares
            WHERE object_id = $1 -- document_id
              AND entity_id = $2 -- authenticated_user_id
              AND entity_type = 'user'
              AND object_type = 'document'
        UNION ALL
            -- Organization shares for user's orgs for this document
            SELECT os.access_level
            FROM object_shares os
            WHERE os.object_id = $1 -- document_id
              AND os.entity_id = ANY($3::UUID[]) -- org_ids (Vec<Uuid>)
              AND os.entity_type = 'organization'
              AND os.object_type = 'document'
        ),
        RankedUserOrgShares AS (
            SELECT
                access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM UserOrgShares
        )
        SELECT
            d.id,
            d.user_id,
            d.title,
            d.content,
            d.sources,
            d.status,
            d.created_at,
            d.updated_at,
            d.is_public,
            d.is_task,
            d.include_research::TEXT as "include_research?",
            (SELECT email FROM users WHERE id = d.user_id) AS "creator_email?",
            (SELECT rus.access_level::TEXT FROM RankedUserOrgShares rus WHERE rus.rn = 1) AS "shared_access_level?"
        FROM
            documents d
        WHERE
            d.id = $1 -- document_id
        "#,
        document_id,           // $1
       authenticated_user_id, // $2
       organization_ids       // $3
   )
   .fetch_optional(&mut **tx)
   .await
}
