//! Finds all active organization memberships for a given user.
use crate::db::organization_members::{OrganizationMember, OrganizationMemberStatus};
use sqlx::{types::Uuid, PgPool};

/// Finds all active organization memberships for a given user.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user.
///
/// # Returns
///
/// A `Result` containing a `Vec<OrganizationMember>` for active memberships,
/// or an `sqlx::Error` on failure.
pub async fn find_active_memberships_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<OrganizationMember>> {
    let active_status = OrganizationMemberStatus::Active.to_string();
    let memberships = sqlx::query_as!(
        OrganizationMember,
        r#"
        SELECT organization_id, user_id, role, status, invited_by_user_id, invited_at, joined_at
        FROM organization_members
        WHERE user_id = $1 AND status = $2
        "#,
        user_id,
        active_status
    )
    .fetch_all(pool)
    .await?;
    Ok(memberships)
} 