//! Revoke an active unlimited access grant for a user.
//!
//! This function soft-deletes an unlimited access grant by setting the
//! revoked_at timestamp and recording who revoked it and why.
//! The grant record remains in the database for audit purposes.
//! Returns the updated grant record.

#![allow(clippy::disallowed_methods)]

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

/// Revoke unlimited access grant for user
#[instrument(skip(pool))]
pub async fn revoke_user_grant(
    pool: &PgPool,
    user_id: Uuid,
    revoked_by_user_id: Uuid,
    revoked_reason: &str,
) -> Result<crate::db::unlimited_access_grant::UnlimitedAccessGrant, sqlx::Error> {
    sqlx::query_as!(
        crate::db::unlimited_access_grant::UnlimitedAccessGrant,
        r#"
        UPDATE unlimited_access_grants
        SET revoked_at = $1,
            revoked_by_user_id = $2,
            revoked_reason = $3,
            updated_at = $1
        WHERE user_id = $4
          AND revoked_at IS NULL
        RETURNING id, user_id, organization_id, granted_at, granted_by_user_id,
                  granted_reason, expires_at, revoked_at, revoked_by_user_id,
                  revoked_reason, notes, metadata, created_at, updated_at
        "#,
        Utc::now(),
        revoked_by_user_id,
        revoked_reason,
        user_id
    )
    .fetch_one(pool)
    .await
}

