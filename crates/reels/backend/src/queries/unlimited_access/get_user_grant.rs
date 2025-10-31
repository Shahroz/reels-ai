//! Get the active unlimited access grant for a user.
//!
//! This function retrieves the active unlimited access grant record
//! for a specific user if one exists. Returns None if no active grant found.
//! Useful for admin interfaces and audit purposes to see grant details.

#![allow(clippy::disallowed_methods)]

use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

/// Get active unlimited access grant for user
#[instrument(skip(pool))]
pub async fn get_user_grant(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<crate::db::unlimited_access_grant::UnlimitedAccessGrant>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::unlimited_access_grant::UnlimitedAccessGrant,
        r#"
        SELECT id, user_id, organization_id, granted_at, granted_by_user_id,
               granted_reason, expires_at, revoked_at, revoked_by_user_id,
               revoked_reason, notes, metadata, created_at, updated_at
        FROM unlimited_access_grants
        WHERE user_id = $1
          AND revoked_at IS NULL
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
}

