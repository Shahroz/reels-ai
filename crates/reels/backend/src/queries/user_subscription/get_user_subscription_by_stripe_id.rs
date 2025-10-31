#![allow(clippy::disallowed_methods)]
//! Get user subscription by Stripe subscription ID.
//!
//! This function retrieves a user subscription record by Stripe subscription ID.

use sqlx::{PgPool, Error};
use tracing::instrument;

use crate::db::user_subscription::{UserSubscription, DbUserSubscription};

/// Get user subscription by Stripe subscription ID
#[instrument(skip(pool))]
pub async fn get_user_subscription_by_stripe_id(pool: &PgPool, stripe_subscription_id: &str) -> Result<Option<UserSubscription>, Error> {

    let result = sqlx::query_as!(
        DbUserSubscription,
        r#"
        SELECT id, user_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
               stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end,
               created_at, updated_at
        FROM user_subscriptions
        WHERE stripe_subscription_id = $1
        "#,
        stripe_subscription_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|db_subscription| db_subscription.into_user_subscription()))
}
