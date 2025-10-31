//! Create a new unlimited access grant for a user.
//!
//! This function creates a new unlimited access grant record in the database
//! for a specific user. The grant can optionally have an expiration date.
//! Returns the created grant record. Used by admin endpoints to grant unlimited access.

#![allow(clippy::disallowed_methods)]

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

/// Create unlimited access grant for user
#[instrument(skip(pool))]
pub async fn create_user_grant(
    pool: &PgPool,
    user_id: Uuid,
    granted_by_user_id: Uuid,
    granted_reason: &str,
    expires_at: Option<DateTime<Utc>>,
    notes: Option<&str>,
) -> Result<crate::db::unlimited_access_grant::UnlimitedAccessGrant, sqlx::Error> {
    sqlx::query_as!(
        crate::db::unlimited_access_grant::UnlimitedAccessGrant,
        r#"
        INSERT INTO unlimited_access_grants (
            user_id, granted_by_user_id, granted_reason, expires_at, notes
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, organization_id, granted_at, granted_by_user_id,
                  granted_reason, expires_at, revoked_at, revoked_by_user_id,
                  revoked_reason, notes, metadata, created_at, updated_at
        "#,
        user_id,
        granted_by_user_id,
        granted_reason,
        expires_at,
        notes
    )
    .fetch_one(pool)
    .await
}

