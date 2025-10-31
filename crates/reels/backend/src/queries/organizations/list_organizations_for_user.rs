//! Lists all organizations a specific user is a member of.
use crate::db::organizations::Organization;
use sqlx::{types::Uuid, PgPool};

/// Lists all organizations a specific user is a member of.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user whose organizations to list.
///
/// # Returns
///
/// A `Result` containing a `Vec<Organization>` on success, or an `sqlx::Error` on failure.
/// The vector may be empty if the user is not part of any organizations.
pub async fn list_organizations_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<Organization>> {
    let orgs = sqlx::query_as!(
        Organization,
        r#"
        SELECT o.id, o.name, o.owner_user_id, o.stripe_customer_id, o.settings, o.is_personal, o.created_at, o.updated_at
        FROM organizations o
        JOIN organization_members om ON o.id = om.organization_id
        WHERE om.user_id = $1 AND om.status = 'active' -- Ensure user is an active member
        ORDER BY o.name ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(orgs)
} 