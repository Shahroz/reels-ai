//! Updates the status and optionally the role of an organization member.
use crate::db::organization_members::{OrganizationMember, OrganizationMemberStatus};
use sqlx::{Postgres, Transaction, types::Uuid};
use chrono::Utc;

/// Updates the status and optionally the role of an organization member.
/// Sets `joined_at` to current time if status changes to 'Active' and was not already set.
pub async fn update_member_status_and_role(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    user_id: Uuid,
    new_status: OrganizationMemberStatus,
    new_role: Option<String>, // Optionally update role
) -> anyhow::Result<OrganizationMember> {
    let status_str = new_status.to_string();
    let now = Utc::now();

    let member = sqlx::query_as!(
        OrganizationMember,
        r#"
        UPDATE organization_members
        SET status = $3,
            role = COALESCE($4, role),
            joined_at = CASE
                            WHEN joined_at IS NULL AND $3 = 'active' THEN $5
                            ELSE joined_at
                        END
        WHERE organization_id = $1 AND user_id = $2
        RETURNING organization_id, user_id, role, status, invited_by_user_id, invited_at, joined_at
        "#,
        org_id,
        user_id,
        status_str,
        new_role,
        now // for joined_at ($5)
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(member)
} 