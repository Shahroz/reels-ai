//! List all unlimited access grants with pagination.
//!
//! This function retrieves unlimited access grants from the database
//! with optional filtering by active/revoked status and pagination support.
//! Used by admin interfaces to view and manage all unlimited access grants.
//! Returns grants ordered by granted_at descending (most recent first).

#![allow(clippy::disallowed_methods)]

use sqlx::PgPool;
use tracing::instrument;

/// List all unlimited access grants with pagination
#[instrument(skip(pool))]
pub async fn list_all_grants(
    pool: &PgPool,
    include_revoked: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::db::unlimited_access_grant::UnlimitedAccessGrant>, sqlx::Error> {
    if include_revoked {
        sqlx::query_as!(
            crate::db::unlimited_access_grant::UnlimitedAccessGrant,
            r#"
            SELECT id, user_id, organization_id, granted_at, granted_by_user_id,
                   granted_reason, expires_at, revoked_at, revoked_by_user_id,
                   revoked_reason, notes, metadata, created_at, updated_at
            FROM unlimited_access_grants
            ORDER BY granted_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as!(
            crate::db::unlimited_access_grant::UnlimitedAccessGrant,
            r#"
            SELECT id, user_id, organization_id, granted_at, granted_by_user_id,
                   granted_reason, expires_at, revoked_at, revoked_by_user_id,
                   revoked_reason, notes, metadata, created_at, updated_at
            FROM unlimited_access_grants
            WHERE revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY granted_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await
    }
}

