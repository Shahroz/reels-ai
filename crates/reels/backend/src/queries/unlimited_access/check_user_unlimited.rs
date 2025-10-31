//! Check if a user has active unlimited access.
//!
//! This function queries the unlimited_access_grants table to determine
//! if a specific user has an active grant (not revoked and not expired).
//! Returns true if unlimited access is active, false otherwise.
//! Used in credit deduction logic to bypass credit checks.

#![allow(clippy::disallowed_methods)]

use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

/// Check if a user has active unlimited access
#[instrument(skip(pool))]
pub async fn check_user_unlimited(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM unlimited_access_grants
            WHERE user_id = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
        ) as "exists!"
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;
    
    Ok(result.exists)
}

