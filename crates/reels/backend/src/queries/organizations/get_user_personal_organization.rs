//! Retrieves a user's personal organization.
//!
//! Every user should have exactly one personal organization (is_personal = true).
//! This query retrieves it by user_id.

/// Gets the personal organization for a user
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `user_id` - The UUID of the user
///
/// # Returns
///
/// A `Result` containing `Option<Organization>` on success, or an `sqlx::Error` on failure.
/// Returns `None` if the user has no personal organization.
#[tracing::instrument(skip(pool))]
pub async fn get_user_personal_organization(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> Result<Option<crate::db::organizations::Organization>, sqlx::Error> {
    let org = sqlx::query_as!(
        crate::db::organizations::Organization,
        r#"
        SELECT id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        FROM organizations
        WHERE owner_user_id = $1 AND is_personal = true
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(org)
}

