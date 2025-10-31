#![allow(clippy::disallowed_methods)]
//! Update user subscription status.
//!
//! This function updates the status of a user subscription.

use sqlx::{PgPool, Error};
use tracing::instrument;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::db::user_subscription::{UserSubscription, DbUserSubscription};

/// Update user subscription status
#[instrument(skip(pool))]
pub async fn update_user_subscription_status(
    pool: &PgPool,
    stripe_subscription_id: &str,
    status: SubscriptionStatus,
) -> Result<UserSubscription, Error> {

    let result = sqlx::query_as!(
        DbUserSubscription,
        r#"
        UPDATE user_subscriptions 
        SET status = $1, updated_at = CURRENT_TIMESTAMP
        WHERE stripe_subscription_id = $2
        RETURNING id, user_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
                  stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end,
                  created_at, updated_at
        "#,
        status.as_str(),
        stripe_subscription_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.into_user_subscription())
}
