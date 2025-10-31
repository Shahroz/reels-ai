#![allow(clippy::disallowed_methods)]
//! Create a new organization subscription.
//!
//! This function creates a new organization subscription record in the database.
//! Follows the same pattern as user subscriptions but for organizations.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::db::organization_subscription::{OrganizationSubscription, DbOrganizationSubscription};

/// Create a new organization subscription
#[instrument(skip(pool))]
pub async fn create_organization_subscription(
    pool: &PgPool,
    organization_id: Uuid,
    stripe_subscription_id: &str,
    stripe_product_id: &str,
    stripe_price_id: &str,
    stripe_plan_type: StripePlanType,
    credits_per_month: i32,
    cost: BigDecimal,
    status: SubscriptionStatus,
    current_period_start: DateTime<Utc>,
    current_period_end: DateTime<Utc>,
) -> Result<OrganizationSubscription, Error> {
    let result = sqlx::query_as!(
        DbOrganizationSubscription,
        r#"
        INSERT INTO organization_subscriptions (
            organization_id, stripe_subscription_id, stripe_product_id, stripe_price_id, 
            stripe_plan_type, credits_per_month, cost, status, current_period_start, current_period_end
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, organization_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
                  stripe_plan_type, credits_per_month, cost, status, current_period_start, current_period_end,
                  created_at, updated_at
        "#,
        organization_id,
        stripe_subscription_id,
        stripe_product_id,
        stripe_price_id,
        stripe_plan_type.as_str(),
        credits_per_month,
        cost,
        status.as_str(),
        current_period_start,
        current_period_end
    )
    .fetch_one(pool)
    .await?;

    Ok(result.into_organization_subscription())
}

