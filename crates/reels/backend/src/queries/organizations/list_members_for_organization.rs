//! Lists all members of a specific organization, including some user details.
use sqlx::{types::Uuid, PgPool};

/// Lists all members of a specific organization, including some user details.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `org_id` - The UUID of the organization whose members to list.
///
/// # Returns
///
/// A `Result` containing a `Vec<OrganizationMemberResponse>` on success,
/// or an `sqlx::Error` on failure.
pub async fn list_members_for_organization(
    pool: &PgPool,
    org_id: Uuid,
) -> anyhow::Result<Vec<crate::routes::organizations::member_response::OrganizationMemberResponse>> {
    let members = sqlx::query_as!(
        crate::routes::organizations::member_response::OrganizationMemberResponse,
        r#"
        SELECT
            om.user_id,
            om.role,
            om.status,
            om.invited_by_user_id,
            om.invited_at,
            om.joined_at,
            u.email,
            NULL AS name -- users table does not have a name column, providing NULL
        FROM
            organization_members om
        JOIN
            users u ON om.user_id = u.id
        WHERE
            om.organization_id = $1
        ORDER BY
            u.email ASC -- Order by email since name is not available
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?;

    log::info!("Raw members data from DB for organization_id {org_id}: {members:?}");

    Ok(members)
} 