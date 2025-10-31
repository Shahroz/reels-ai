#![allow(clippy::disallowed_methods)]
//! Get user's subscriptions by status.
//!
//! This function retrieves all user subscriptions with a specific status.

use sqlx::{PgPool, Error};
use tracing::instrument;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::db::user_subscription::{UserSubscription, DbUserSubscription};

/// Get user's subscriptions by status
#[instrument(skip(pool))]
pub async fn get_subscriptions_by_status(pool: &PgPool, status: SubscriptionStatus) -> Result<Vec<UserSubscription>, Error> {

    let results = sqlx::query_as!(
        DbUserSubscription,
        r#"
        SELECT id, user_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
               stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end,
               created_at, updated_at
        FROM user_subscriptions
        WHERE status = $1
        ORDER BY created_at DESC
        "#,
        status.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|db_subscription| db_subscription.into_user_subscription())
        .collect())
}
