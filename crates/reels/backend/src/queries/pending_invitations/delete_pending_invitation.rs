//! Deletes a pending invitation by its ID.
//!
//! This function removes a record from the `pending_invitations` table
//! based on its unique identifier. This is typically done after a user
//! has successfully accepted an invitation and joined the organization.
//! Returns the number of rows affected.

use sqlx::PgConnection;
use uuid::Uuid;

pub async fn delete_pending_invitation(
    executor: &mut PgConnection,
    invitation_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM pending_invitations WHERE id = $1",
        invitation_id
    )
        .execute(executor)
        .await?;

    Ok(result.rows_affected())
}