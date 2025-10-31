//! Finds a pending invitation by organization ID and invited email.
//!
//! This is useful for checking if an invitation already exists for a
//! specific email address within an organization before creating a new one,
//! preventing duplicate invitations.
//! Returns `None` if no matching invitation is found.

pub async fn find_pending_invitation_by_org_and_email(
    pool: &sqlx::postgres::PgPool,
    organization_id: sqlx::types::Uuid,
    invited_email: &str,
) -> Result<Option<crate::db::pending_invitations::PendingInvitation>, sqlx::Error> {
    let invitation = sqlx::query_as!(
        crate::db::pending_invitations::PendingInvitation,
        r#"
        SELECT
            id, organization_id, invited_email, role_to_assign, invitation_token,
            token_expires_at, invited_by_user_id, created_at, updated_at
        FROM pending_invitations
        WHERE organization_id = $1 AND invited_email = $2
        "#,
        organization_id,
        invited_email
    )
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}