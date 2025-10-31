#![allow(clippy::disallowed_methods)]
//! Get user's current active subscription.
//!
//! This function retrieves the current active subscription for a user.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::db::user_subscription::{UserSubscription, DbUserSubscription};

/// Get user's current active subscription
#[instrument(skip(pool))]
pub async fn get_current_active_subscription(pool: &PgPool, user_id: Uuid) -> Result<Option<UserSubscription>, Error> {

    let results = sqlx::query_as!(
        DbUserSubscription,
        r#"
        SELECT id, user_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
               stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end,
               created_at, updated_at
        FROM user_subscriptions
        WHERE user_id = $1 
          AND status = ANY($2)
        ORDER BY created_at DESC
        "#,
        user_id,
        &[
            SubscriptionStatus::Active.as_str(),
            SubscriptionStatus::Trialing.as_str(),
            SubscriptionStatus::Trial.as_str(),
        ] as &[&str]
    )
    .fetch_optional(pool)
    .await?;

    Ok(results.map(|db_subscription| db_subscription.into_user_subscription()))
}
