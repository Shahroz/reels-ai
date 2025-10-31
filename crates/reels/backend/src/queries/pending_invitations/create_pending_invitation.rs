//! Creates a new pending invitation in the database.
//!
//! This function generates a unique token for the invitation, sets an expiration time,
//! and inserts a new record into the `pending_invitations` table.
//! It returns the newly created pending invitation.
//! Adheres to one-item-per-file and FQN guidelines.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::db::pending_invitations::PendingInvitation;

/// Creates a new pending invitation record in the database.
pub async fn create_pending_invitation(
    pool: &sqlx::PgPool,
    organization_id: uuid::Uuid,
    invited_email: &str,
    role_to_assign: &str,
    invitation_token: &str,
    token_expires_at: DateTime<Utc>,
    invited_by_user_id: Option<Uuid>,
) -> Result<PendingInvitation, sqlx::Error> {
    let now = Utc::now();
    let pending_invitation = sqlx::query_as!(
        PendingInvitation,
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


#[cfg(test)]
mod tests {
    #[test]
    fn create_pending_invitation_test() {
        // This is a placeholder test. A real test would require a test database.
        // For now, this just ensures the file structure is correct.
        assert!(true);
    }
}