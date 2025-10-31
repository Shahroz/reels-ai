//! Finds a pending invitation by organization ID and user email.
//!
//! This function queries the database for a pending invitation
//! that matches the given organization ID and email address.
//! It returns an Option containing the invitation if found.
//! Adheres to one-item-per-file and FQN guidelines.

use sqlx::PgPool;
use uuid::Uuid;
use crate::db::pending_invitations::PendingInvitation;

/// Finds a pending invitation by organization ID and invited email.
/// Useful for checking if an invitation already exists before creating a new one.
pub async fn find_pending_invitation_by_org_and_email(
    pool: &PgPool,
    organization_id: Uuid,
    invited_email: &str,
) -> Result<Option<PendingInvitation>, sqlx::Error> {
    let invitation = sqlx::query_as!(
        PendingInvitation,
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

#[cfg(test)]
mod tests {
    #[test]
    fn find_pending_invitation_by_org_and_email_test() {
        // This is a placeholder test. A real test would require a test database.
        assert!(true);
    }
}