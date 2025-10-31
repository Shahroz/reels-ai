//! Creates a new pending invitation record in the database.
//!
//! This function inserts a new row into the `pending_invitations` table
//! with the details of the invitation, such as the organization,
//! the invited user's email, the role to assign, and the expiration date.
//! It returns the newly created invitation record.

pub async fn create_pending_invitation(
    pool: &sqlx::postgres::PgPool,
    organization_id: sqlx::types::Uuid,
    invited_email: &str,
    role_to_assign: &str,
    invitation_token: &str,
    token_expires_at: chrono::DateTime<chrono::Utc>,
    invited_by_user_id: Option<sqlx::types::Uuid>,
) -> Result<crate::db::pending_invitations::PendingInvitation, sqlx::Error> {
    let now = chrono::Utc::now();
    let pending_invitation = sqlx::query_as!(
        crate::db::pending_invitations::PendingInvitation,
        r#"
        INSERT INTO pending_invitations (
            organization_id, invited_email, role_to_assign, invitation_token,
            token_expires_at, invited_by_user_id, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id, organization_id, invited_email, role_to_assign, invitation_token,
            token_expires_at, invited_by_user_id, created_at, updated_at
        "#,
        organization_id,
        invited_email,
        role_to_assign,
        invitation_token,
        token_expires_at,
        invited_by_user_id,
        now, // created_at
        now  // updated_at
    )
    .fetch_one(pool)
    .await?;

    Ok(pending_invitation)
}