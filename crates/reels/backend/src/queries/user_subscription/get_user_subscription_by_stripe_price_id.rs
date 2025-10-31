#![allow(clippy::disallowed_methods)]
//! Get user subscription by Stripe price ID.
//!
//! This function retrieves a user subscription record by user ID and Stripe price ID.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::user_subscription::{UserSubscription, DbUserSubscription};

/// Get user's subscriptions by stripe price id
#[instrument(skip(pool))]
pub async fn get_user_subscription_by_stripe_price_id(pool: &PgPool, user_id: Uuid, price_id: &str) -> Result<Option<UserSubscription>, Error> {

    let results = sqlx::query_as!(
        DbUserSubscription,
        r#"
        SELECT id, user_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
               stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end,
               created_at, updated_at
        FROM user_subscriptions
        WHERE user_id = $1 AND stripe_price_id = $2
        ORDER BY created_at DESC LIMIT 1
        "#,
        user_id,
        price_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(results.map(|db_subscription| db_subscription.into_user_subscription()))
}
