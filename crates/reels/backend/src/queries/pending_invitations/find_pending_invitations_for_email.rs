//! Finds all pending invitations for a given email.
//!
//! This function retrieves all invitations sent to a specific email address,
//! joining with the organizations and users tables to include the organization's
//! name and the inviter's email address.
//! The results are ordered by creation date.

pub async fn find_pending_invitations_for_email(
    pool: &sqlx::postgres::PgPool,
    email: &str,
) -> Result<Vec<crate::db::pending_invitations::PendingInvitationResponse>, sqlx::Error> {
    let invitations = sqlx::query_as!(
        crate::db::pending_invitations::PendingInvitationResponse,
        r#"
        SELECT
            pi.id AS pending_invitation_id,
            pi.organization_id,
            org.name AS organization_name,
            pi.invited_email,
            pi.role_to_assign,
            pi.invitation_token,
            pi.token_expires_at,
            pi.invited_by_user_id,
            inviter.email AS inviter_email,
            pi.created_at
        FROM
            pending_invitations pi
        JOIN
            organizations org ON pi.organization_id = org.id
        LEFT JOIN
            users inviter ON pi.invited_by_user_id = inviter.id
        WHERE
            pi.invited_email = $1
        ORDER BY
            pi.created_at DESC
        "#,
        email
    )
    .fetch_all(pool)
    .await?;

    Ok(invitations)
}