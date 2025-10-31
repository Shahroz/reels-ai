//! Creates a personal organization for a user.
//!
//! Creates a personal organization, adds the user as owner member,
//! and initializes organization credit allocation. All operations are
//! performed within a single transaction to ensure atomicity.
//! Personal organizations are identified by is_personal = true.

/// Creates a personal organization for a user with initial credits
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `user_id` - The UUID of the user
/// * `user_email` - The user's email (used to generate org name)
/// * `initial_credits` - Initial credit balance for the organization
///
/// # Returns
///
/// A `Result` containing the created `Organization` on success, or an `sqlx::Error` on failure

use bigdecimal::BigDecimal;

#[tracing::instrument(skip(pool))]
pub async fn create_personal_organization(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    user_email: &str,
    initial_credits: i32,
) -> Result<crate::db::organizations::Organization, sqlx::Error> {
    // Start transaction
    let mut tx = pool.begin().await?;
    
    // Create the personal organization
    let org_name = format!("{}'s Personal Workspace", user_email);
    let org = sqlx::query_as!(
        crate::db::organizations::Organization,
        r#"
        INSERT INTO organizations (name, owner_user_id, is_personal)
        VALUES ($1, $2, true)
        RETURNING id, name, owner_user_id, stripe_customer_id, settings, is_personal, created_at, updated_at
        "#,
        org_name,
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Add user as owner member
    sqlx::query!(
        r#"
        INSERT INTO organization_members (organization_id, user_id, role, status, invited_at, joined_at)
        VALUES ($1, $2, 'owner', 'active', NOW(), NOW())
        "#,
        org.id,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    
    // Create organization credit allocation
    sqlx::query!(
        r#"
        INSERT INTO organization_credit_allocation (organization_id, credits_remaining, last_reset_date)
        VALUES ($1, $2, NOW())
        "#,
        org.id,
        BigDecimal::from(initial_credits)
    )
    .execute(&mut *tx)
    .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    log::info!(
        "Created personal organization {} for user {} with {} credits",
        org.id,
        user_id,
        initial_credits.to_string()
    );
    
    Ok(org)
}

