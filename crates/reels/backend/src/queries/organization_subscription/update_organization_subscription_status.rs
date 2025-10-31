#![allow(clippy::disallowed_methods)]
//! Update organization subscription status.
//!
//! This function updates the status of an organization subscription.
//! Used for handling subscription lifecycle events.

use sqlx::{PgPool, Error};
use tracing::instrument;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::db::organization_subscription::{OrganizationSubscription, DbOrganizationSubscription};

/// Update organization subscription status
#[instrument(skip(pool))]
pub async fn update_organization_subscription_status(
    pool: &PgPool,
    stripe_subscription_id: &str,
    status: SubscriptionStatus
) -> Result<OrganizationSubscription, Error> {
    let result = sqlx::query_as!(
        DbOrganizationSubscription,
        r#"
        UPDATE organization_subscriptions
        SET status = $1, updated_at = CURRENT_TIMESTAMP
        WHERE stripe_subscription_id = $2
        RETURNING id, organization_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
                  stripe_plan_type, credits_per_month, cost, status, current_period_start, current_period_end,
                  created_at, updated_at
        "#,
        status.as_str(),
        stripe_subscription_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.into_organization_subscription())
}

