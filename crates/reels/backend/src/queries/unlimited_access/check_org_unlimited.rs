//! Check if an organization has active unlimited access.
//!
//! This function queries the unlimited_access_grants table to determine
//! if a specific organization has an active grant (not revoked and not expired).
//! Returns true if unlimited access is active, false otherwise.
//! Used in credit deduction logic to bypass credit checks for organizations.

#![allow(clippy::disallowed_methods)]

use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

/// Check if an organization has active unlimited access
#[instrument(skip(pool))]
pub async fn check_org_unlimited(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM unlimited_access_grants
            WHERE organization_id = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
        ) as "exists!"
        "#,
        organization_id
    )
    .fetch_one(pool)
    .await?;
    
    Ok(result.exists)
}

