//! Finds a pending invitation by its unique token.
//!
//! This function queries the `pending_invitations` table for a record
//! that matches the provided invitation token. It is used to validate
//! an invitation token when a user attempts to accept an invitation.
//! Returns `None` if no matching invitation is found.

pub async fn find_pending_invitation_by_token(
    pool: &sqlx::postgres::PgPool,
    token: &str,
) -> Result<Option<crate::db::pending_invitations::PendingInvitation>, sqlx::Error> {
    let invitation = sqlx::query_as!(
        crate::db::pending_invitations::PendingInvitation,
        r#"
        SELECT
            id, organization_id, invited_email, role_to_assign, invitation_token,
            token_expires_at, invited_by_user_id, created_at, updated_at
        FROM pending_invitations
        WHERE invitation_token = $1
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}