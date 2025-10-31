//! Removes a member from an organization.
use sqlx::{types::Uuid, PgPool};

/// Removes a member from an organization.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `org_id` - The UUID of the organization.
/// * `user_id` - The UUID of the user to remove.
///
/// # Returns
///
/// A `Result` containing the number of rows affected (should be 1 on success)
/// or an `sqlx::Error` on failure.
pub async fn remove_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> anyhow::Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM organization_members WHERE organization_id = $1 AND user_id = $2",
        org_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
} 