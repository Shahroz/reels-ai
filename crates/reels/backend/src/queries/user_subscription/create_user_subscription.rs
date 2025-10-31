#![allow(clippy::disallowed_methods)]
//! Create a new user subscription.
//!
//! This function creates a new user subscription record in the database.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::db::user_subscription::{UserSubscription, DbUserSubscription};

/// Create a new user subscription
#[instrument(skip(pool))]
pub async fn create_user_subscription(
    pool: &PgPool,
    user_id: Uuid,
    stripe_subscription_id: &str,
    stripe_product_id: &str,
    stripe_price_id: &str,
    stripe_plan_id: &str,
    stripe_plan_type: StripePlanType,
    credits: i32,
    cost: BigDecimal,
    status: SubscriptionStatus,
    current_period_start: DateTime<Utc>,
    current_period_end: DateTime<Utc>,
) -> Result<UserSubscription, Error> {
    let result = sqlx::query_as!(
        DbUserSubscription,
        r#"
        INSERT INTO user_subscriptions (
            user_id, stripe_subscription_id, stripe_product_id, stripe_price_id, 
            stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, user_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
                  stripe_plan_id, stripe_plan_type, credits, cost, status, current_period_start, current_period_end,
                  created_at, updated_at
        "#,
        user_id,
        stripe_subscription_id,
        stripe_product_id,
        stripe_price_id,
        stripe_plan_id,
        stripe_plan_type.as_str(),
        credits,
        cost,
        status.as_str(),
        current_period_start,
        current_period_end
    )
    .fetch_one(pool)
    .await?;

    Ok(result.into_user_subscription())
}
