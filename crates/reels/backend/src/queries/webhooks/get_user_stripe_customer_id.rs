//! Query to get a user's Stripe customer ID.
//!
//! This function retrieves the Stripe customer ID for a given user.
//! Used during checkout to check if the user already has a Stripe customer.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// Gets a user's Stripe customer ID.
///
/// # Arguments
/// * `pool` - A reference to the PostgreSQL connection pool.
/// * `user_id` - The ID of the user.
///
/// # Returns
/// `Ok(Some(String))` if the user has a Stripe customer ID, `Ok(None)` otherwise.
/// Returns `Err` if a database error occurs.
#[instrument(skip(pool))]
pub async fn get_user_stripe_customer_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<String>> {
    let result = sqlx::query!(
        "SELECT stripe_customer_id FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.and_then(|r| r.stripe_customer_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_get_user_stripe_customer_id_with_customer(pool: PgPool) -> Result<()> {
        let user_id = Uuid::new_v4();
        let customer_id = "cus_test123";

        // Create a test user with a Stripe customer ID
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash, stripe_customer_id) VALUES ($1, $2, $3, $4)",
            user_id,
            "test@example.com",
            "hash",
            customer_id
        )
        .execute(&pool)
        .await?;

        let result = get_user_stripe_customer_id(&pool, user_id).await?;
        assert_eq!(result, Some(customer_id.to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_stripe_customer_id_without_customer(pool: PgPool) -> Result<()> {
        let user_id = Uuid::new_v4();

        // Create a test user without a Stripe customer ID
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
            user_id,
            "test2@example.com",
            "hash2"
        )
        .execute(&pool)
        .await?;

        let result = get_user_stripe_customer_id(&pool, user_id).await?;
        assert_eq!(result, None);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_stripe_customer_id_nonexistent(pool: PgPool) -> Result<()> {
        let nonexistent_id = Uuid::new_v4();

        let result = get_user_stripe_customer_id(&pool, nonexistent_id).await?;
        assert_eq!(result, None);

        Ok(())
    }
}

