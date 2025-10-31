//! Query to update an organization's Stripe customer ID.
//!
//! This function is used when creating a Stripe customer for an organization
//! during the checkout process. It supports lazy creation of Stripe customers
//! for organizations that don't have one yet.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// Updates the Stripe customer ID for an organization.
///
/// This is typically called during the checkout process when an organization
/// doesn't have a Stripe customer yet and one needs to be created.
///
/// # Arguments
/// * `pool` - A reference to the PostgreSQL connection pool.
/// * `organization_id` - The ID of the organization to update.
/// * `stripe_customer_id` - The Stripe customer ID to assign.
///
/// # Returns
/// `Ok(())` if successful, `Err` if the update fails.
#[instrument(skip(pool))]
pub async fn update_organization_stripe_customer_id(
    pool: &PgPool,
    organization_id: Uuid,
    stripe_customer_id: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE organizations
        SET stripe_customer_id = $1,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        "#,
        stripe_customer_id,
        organization_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_update_organization_stripe_customer_id(pool: PgPool) -> Result<()> {
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let stripe_customer_id = "cus_test123";

        // Create a test user first (required for foreign key constraint)
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            owner_id,
            "test@example.com"
        )
        .execute(&pool)
        .await?;

        // Create a test organization
        sqlx::query!(
            "INSERT INTO organizations (id, name, owner_user_id) VALUES ($1, $2, $3)",
            org_id,
            "Test Org",
            owner_id
        )
        .execute(&pool)
        .await?;

        // Update the Stripe customer ID
        update_organization_stripe_customer_id(&pool, org_id, stripe_customer_id).await?;

        // Verify the update
        let org = sqlx::query!(
            "SELECT stripe_customer_id FROM organizations WHERE id = $1",
            org_id
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(org.stripe_customer_id, Some(stripe_customer_id.to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_nonexistent_organization(pool: PgPool) -> Result<()> {
        let nonexistent_id = Uuid::new_v4();
        let stripe_customer_id = "cus_test456";

        // Attempt to update a non-existent organization (should not error, just update 0 rows)
        update_organization_stripe_customer_id(&pool, nonexistent_id, stripe_customer_id).await?;

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_organization_stripe_customer_id_twice(pool: PgPool) -> Result<()> {
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let first_customer_id = "cus_first";
        let second_customer_id = "cus_second";

        // Create a test user first (required for foreign key constraint)
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            owner_id,
            "test2@example.com"
        )
        .execute(&pool)
        .await?;

        // Create a test organization
        sqlx::query!(
            "INSERT INTO organizations (id, name, owner_user_id) VALUES ($1, $2, $3)",
            org_id,
            "Test Org 2",
            owner_id
        )
        .execute(&pool)
        .await?;

        // Update with first customer ID
        update_organization_stripe_customer_id(&pool, org_id, first_customer_id).await?;

        // Update with second customer ID
        update_organization_stripe_customer_id(&pool, org_id, second_customer_id).await?;

        // Verify the final update
        let org = sqlx::query!(
            "SELECT stripe_customer_id FROM organizations WHERE id = $1",
            org_id
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(org.stripe_customer_id, Some(second_customer_id.to_string()));

        Ok(())
    }
}

