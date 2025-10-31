#![allow(clippy::disallowed_methods)]
//! Get organization subscription by Stripe subscription ID.
//!
//! This function retrieves an organization subscription record by Stripe subscription ID.
//! Used primarily for webhook processing.

use sqlx::{PgPool, Error};
use tracing::instrument;

use crate::db::organization_subscription::{OrganizationSubscription, DbOrganizationSubscription};

/// Get organization subscription by Stripe subscription ID
#[instrument(skip(pool))]
pub async fn get_organization_subscription_by_stripe_id(
    pool: &PgPool,
    stripe_subscription_id: &str
) -> Result<Option<OrganizationSubscription>, Error> {
    let result = sqlx::query_as!(
        DbOrganizationSubscription,
        r#"
        SELECT id, organization_id, stripe_subscription_id, stripe_product_id, stripe_price_id,
               stripe_plan_type, credits_per_month, cost, status, current_period_start, current_period_end,
               created_at, updated_at
        FROM organization_subscriptions
        WHERE stripe_subscription_id = $1
        "#,
        stripe_subscription_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|db_subscription| db_subscription.into_organization_subscription()))
}

