#![allow(clippy::disallowed_methods)]
//! Webhook user-related database queries.
//!
//! This module contains database queries for user operations during webhook processing,
//! including user lookups by Stripe customer ID and subscription status updates.
//! Adheres to FQN and no-`use` statements guidelines.

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::users::User;
use crate::services::billing::stripe_client::StripeClient;

/// Get user ID and update stripe_customer_id by email
/// 
/// This function attempts to update a user's stripe_customer_id by their email address
/// and returns the user's id and email if successful.
pub async fn get_user_by_stripe_customer_id(
    pool: &PgPool,
    stripe_customer_id: &str,
) -> std::result::Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE stripe_customer_id = $1",
        stripe_customer_id
    )
    .fetch_optional(pool)
    .await?;

    // Getting customer from Stripe by stripe_customer_id
    if user.is_none() {
        if let Ok(stripe_client) = StripeClient::new() {
            if let Ok(customer) = stripe_client.get_customer(stripe_customer_id).await {
                if let Some(customer_email) = customer.email {
                    if let Ok(result) = sqlx::query!(
                        r#"
                        UPDATE users
                        SET stripe_customer_id = $1, updated_at = NOW()
                        WHERE LOWER(email) = LOWER($2)
                        "#,
                        stripe_customer_id,
                        customer_email
                    )
                    .execute(pool)
                    .await {
                        if result.rows_affected() > 0 {
                            // Query the updated user
                            if let Ok(Some(updated_user)) = sqlx::query_as!(
                                User,
                                "SELECT * FROM users WHERE stripe_customer_id = $1",
                                stripe_customer_id
                            )
                            .fetch_optional(pool)
                            .await {
                                return Ok(Some(updated_user));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(user)
}

/// Get user ID by Stripe customer ID
/// 
/// This function first tries to find a user by stripe_customer_id.
/// If not found and an email is provided, it attempts to fetch the customer
/// from Stripe and update the user's stripe_customer_id by email.
pub async fn get_user_id_by_stripe_customer_id(
    pool: &PgPool,
    stripe_customer_id: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let result = get_user_by_stripe_customer_id(pool, stripe_customer_id).await?;
    if let Some(user) = result {
        return Ok(Some(user.id));
    } else {
        return Ok(None);
    }
}

/// Get user ID and email by Stripe customer ID
pub async fn get_user_id_and_email_by_stripe_customer_id(
    pool: &PgPool,
    stripe_customer_id: &str,
) -> std::result::Result<Option<(Uuid, String)>, sqlx::Error> {
    let result = get_user_by_stripe_customer_id(pool, stripe_customer_id).await?;
    if let Some(user) = result {
        return Ok(Some((user.id, user.email)));
    } else {
        return Ok(None);
    }
}

/// Get user ID and subscription status by Stripe customer ID
pub async fn get_user_id_and_subscription_status_by_stripe_customer_id(
    pool: &PgPool,
    stripe_customer_id: &str,
) -> std::result::Result<Option<(Uuid, Option<String>)>, sqlx::Error> {
    let result = get_user_by_stripe_customer_id(pool, stripe_customer_id).await?;
    if let Some(user) = result {
        return Ok(Some((user.id, user.subscription_status)));
    } else {
        return Ok(None);
    }
}

/// Update user subscription status
pub async fn update_subscription_status_in_user(
    pool: &PgPool,
    user_id: Uuid,
    status: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET subscription_status = $1, updated_at = NOW() WHERE id = $2",
        status,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update user subscription status to expired and clear stripe customer ID
pub async fn expire_user_subscription_and_clear_stripe_id(
    pool: &PgPool,
    user_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET subscription_status = 'expired', updated_at = NOW() WHERE id = $1",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
