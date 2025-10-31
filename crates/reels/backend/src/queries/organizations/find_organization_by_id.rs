//! Finds an organization by its unique ID.
use crate::db::organizations::Organization;
use sqlx::{types::Uuid, PgPool};

/// Finds an organization by its unique ID.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `org_id` - The UUID of the organization to find.
///
/// # Returns
///
/// A `Result` containing an `Option<Organization>` on success, or an `sqlx::Error` on failure.
/// The `Option` is `Some(Organization)` if found, `None` otherwise.
pub async fn find_organization_by_id(
    pool: &PgPool,
    org_id: Uuid,
) -> anyhow::Result<Option<Organization>> {
    let org = sqlx::query_as!(
        Organization,
        r#"
        SELECT id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
        org_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(org)
} 