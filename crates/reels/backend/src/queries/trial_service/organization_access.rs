#![allow(clippy::disallowed_methods)]
//! **DEPRECATED:** Trial service organization access database queries.
//!
//! This module is deprecated as of 2025-10-17. The organization membership "hack" that granted
//! access based solely on membership in organizations with paid owners has been removed.
//! Access now requires individual credits, trial status, or active subscription.
//!
//! This file is kept for reference and potential rollback capability but should not be used
//! in new code. The function has been marked with #[deprecated] attribute.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Deprecated - organization membership hack removed
//! - [Prior updates not documented in original file]

use sqlx::PgPool;
use uuid::Uuid;

/// **DEPRECATED:** Check if user has organization access.
///
/// This function is deprecated as of 2025-10-17. Organization membership no longer grants
/// automatic access. Users must have individual credits, trial status, or active subscription.
///
/// # Deprecation Note
/// This was part of a legacy "hack" that granted full access to any user who was a member
/// of an organization whose owner had a paid subscription. This undermined the credit system
/// and has been removed. Investigation showed only 1 dormant user (0 active users) would be
/// impacted by the removal.
#[deprecated(
    since = "1.0.0",
    note = "Organization membership hack removed as of 2025-10-17. Use credit-based access instead."
)]
pub async fn has_user_organization_access(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let has_access: Option<bool> = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 
            FROM organizations o
            INNER JOIN organization_members om ON o.id = om.organization_id
            INNER JOIN users owner ON owner.id = o.owner_user_id
            WHERE om.user_id = $1 
            AND om.status = 'active'
            AND owner.subscription_status IN ('active', 'cancelled')
            AND owner.stripe_customer_id IS NOT NULL
        ) as has_access
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(has_access.unwrap_or(false))
}
