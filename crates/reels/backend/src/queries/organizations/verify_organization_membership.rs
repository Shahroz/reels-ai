//! Verify if a user is a member of an organization.
//!
//! This query is used primarily by route handlers that need to authorize
//! organization-level actions, particularly credit deduction operations.
//! It checks if the user exists in the organization_members table for the given organization.
//! This provides the authorization layer before allowing credit operations to proceed.

use sqlx::PgPool;
use uuid::Uuid;

/// Verify if a user is a member of an organization
///
/// Returns true if the user is a member of the organization, false otherwise.
/// This is used for authorization checks before allowing organization credit operations.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - The user ID to check
/// * `organization_id` - The organization ID to check membership in
///
/// # Example Authorization Flow
/// ```rust,ignore
/// // In a route handler:
/// if let Some(org_id) = request.organization_id {
///     // Verify user is member before allowing org credit deduction
///     if !verify_organization_membership(pool, user_id, org_id).await? {
///         return Err(ErrorResponse::forbidden("Not a member of this organization"));
///     }
/// }
/// ```
#[tracing::instrument(skip(pool))]
pub async fn verify_organization_membership(
    pool: &PgPool,
    user_id: Uuid,
    organization_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 
            FROM organization_members 
            WHERE user_id = $1 
            AND organization_id = $2
        ) as "exists!"
        "#,
        user_id,
        organization_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Verifies the module compiles correctly
        assert!(true);
    }
}

