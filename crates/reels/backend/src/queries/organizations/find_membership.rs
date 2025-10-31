//! Finds a specific membership record by organization and user ID.
use crate::db::organization_members::OrganizationMember;
use sqlx::{Postgres, Transaction, types::Uuid};

/// Finds a specific membership record by organization and user ID.
///
/// # Arguments
///
/// * `tx` - The database transaction.
/// * `org_id` - The UUID of the organization.
/// * `user_id` - The UUID of the user.
///
/// # Returns
///
/// A `Result` containing an `Option<OrganizationMember>` on success, or an `sqlx::Error` on failure.
/// The `Option` is `Some(OrganizationMember)` if the membership exists, `None` otherwise.
pub async fn find_membership(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<Option<OrganizationMember>> {
    let membership = sqlx::query_as!(
        OrganizationMember,
        r#"
        SELECT organization_id, user_id, role, status, invited_by_user_id, invited_at, joined_at
        FROM organization_members
        WHERE organization_id = $1 AND user_id = $2
        "#,
        org_id,
        user_id
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(membership)
} 