//! Adds a new member to an organization or updates an existing invitation.
use crate::db::organization_members::OrganizationMember;
use sqlx::{Postgres, Transaction, types::Uuid};
use chrono::Utc;

/// Adds a new member to an organization or updates an existing invitation.
///
/// If an invitation already exists (status='invited'), it updates the role,
/// status (e.g., to 'active'), invited_by, and sets joined_at.
/// If no record exists, it inserts a new member record.
///
/// # Arguments
///
/// * `tx` - The database transaction.
/// * `org_id` - The UUID of the organization.
/// * `user_id` - The UUID of the user to add.
/// * `role` - The role assigned to the user (e.g., "member", "admin").
/// * `status` - The status of the membership (e.g., "active", "invited").
/// * `invited_by` - Optional UUID of the user who invited this member.
///
/// # Returns
///
/// A `Result` containing the created or updated `OrganizationMember` on success,
/// or an `sqlx::Error` on failure.
pub async fn add_member(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    user_id: Uuid,
    role: &str,
    status: &str,
    invited_by: Option<Uuid>,
) -> anyhow::Result<OrganizationMember> {
    let now = Utc::now();
    let joined_at = if status == "active" { Some(now) } else { None };
    // For new invites or updates where status is 'invited', invited_at should be set/updated.
    // If status is not 'invited' (e.g. directly adding an 'active' member without prior invite step),
    // invited_at might be None or retain its old value if this function also updates.
    // Current logic sets invited_at to now on any add_member call.
    let invited_at = Some(now);

    let member = sqlx::query_as!(
        OrganizationMember,
        r#"
        INSERT INTO organization_members (organization_id, user_id, role, status, invited_by_user_id, invited_at, joined_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (organization_id, user_id) DO UPDATE
        SET role = EXCLUDED.role,
            status = EXCLUDED.status,
            invited_by_user_id = EXCLUDED.invited_by_user_id,
            -- Only update joined_at if transitioning to active or already active
            joined_at = CASE
                          WHEN organization_members.status != 'active' AND EXCLUDED.status = 'active' THEN EXCLUDED.joined_at
                          ELSE organization_members.joined_at
                        END,
            invited_at = EXCLUDED.invited_at -- Always update invited_at on conflict as well, as it's part of EXCLUDED
        RETURNING organization_id, user_id, role, status, invited_by_user_id, invited_at, joined_at
        "#,
        org_id,
        user_id,
        role,
        status,
        invited_by,
        invited_at,
        joined_at
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(member)
} 