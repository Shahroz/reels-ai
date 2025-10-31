//! Query to fetch all organizations a user belongs to along with credit information.
//!
//! This query is used by the GET /api/users/me endpoint to return a complete picture
//! of the user's available credit contexts (personal + all organizations they can access).
//! It performs a join between organization_members, organizations, and organization_credit_allocation.

use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Organization credit information for a user
///
/// Contains the organization's basic info, the user's role, the organization's credit balance,
/// and whether it's a personal organization.
#[derive(Debug, sqlx::FromRow)]
pub struct UserOrganizationWithCredits {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub user_role: String,
    pub credits_remaining: Option<BigDecimal>,
    pub is_personal: bool,
}

/// Get all organizations a user belongs to along with their credit allocations
///
/// Returns a list of organizations with:
/// - Organization ID and name
/// - User's role in the organization
/// - Organization's remaining credits (0 if no allocation exists)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - The user ID to fetch organizations for
///
/// # Returns
/// A vector of organizations with credit information. Empty vector if user has no memberships.
///
/// # Example
/// ```rust,ignore
/// let orgs = get_user_organizations_with_credits(pool, user_id).await?;
/// for org in orgs {
///     println!("Org: {}, Credits: {}", org.organization_name, org.credits_remaining.unwrap_or(0));
/// }
/// ```
#[tracing::instrument(skip(pool))]
pub async fn get_user_organizations_with_credits(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UserOrganizationWithCredits>, sqlx::Error> {
    let results = sqlx::query_as!(
        UserOrganizationWithCredits,
        r#"
        SELECT 
            o.id as organization_id,
            o.name as organization_name,
            om.role as user_role,
            COALESCE(oca.credits_remaining, 0) as credits_remaining,
            o.is_personal as "is_personal!"
        FROM organization_members om
        INNER JOIN organizations o ON om.organization_id = o.id
        LEFT JOIN organization_credit_allocation oca ON o.id = oca.organization_id
        WHERE om.user_id = $1
        ORDER BY o.is_personal DESC, o.name ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Verifies the module compiles correctly
        assert!(true);
    }
}

