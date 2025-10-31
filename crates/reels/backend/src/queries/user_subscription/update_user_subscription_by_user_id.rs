#![allow(clippy::disallowed_methods)]
//! Update user's subscription by user ID with flexible field updates.
//!
//! This function updates a user subscription with flexible field updates.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::user_subscription::{UserSubscription};
use crate::schemas::user_subscription_schemas::UserSubscriptionUpdates;

/// Update user's subscription by user ID with flexible field updates
#[instrument(skip(pool))]
pub async fn update_user_subscription_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
    user_subscription: Option<UserSubscription>,
    updates: UserSubscriptionUpdates,
) -> Result<UserSubscription, Error> {
    // Use a simple approach with individual UPDATE queries for each field
    // This is more maintainable and type-safe than dynamic SQL building
    let mut user_subscription = if let Some(sub) = user_subscription {
        sub
    } else {
        match crate::queries::user_subscription::get_user_subscription_by_user_id::get_user_subscription_by_user_id(pool, user_id).await {
            Ok(Some(sub)) => sub,
            _ => {
                log::error!("Failed to get user subscription by user id: {user_id}");
                return Err(Error::RowNotFound)
            },
        }
    };
    
    if let Some(ref stripe_subscription_id) = updates.stripe_subscription_id {
        user_subscription.stripe_subscription_id = stripe_subscription_id.clone();
    }
    
    if let Some(ref stripe_product_id) = updates.stripe_product_id {
        user_subscription.stripe_product_id = stripe_product_id.clone();
    }
    
    if let Some(ref stripe_price_id) = updates.stripe_price_id {
        user_subscription.stripe_price_id = stripe_price_id.clone();
    }
    
    if let Some(ref stripe_plan_id) = updates.stripe_plan_id {
        user_subscription.stripe_plan_id = stripe_plan_id.clone();
    }
    
    if let Some(ref stripe_plan_type) = updates.stripe_plan_type {
        user_subscription.stripe_plan_type = stripe_plan_type.clone();
    }
    
    if let Some(credits) = updates.credits {
        user_subscription.credits = credits;
    }
    
    if let Some(ref cost) = updates.cost {
        user_subscription.cost = cost.clone();
    }
    
    if let Some(status) = updates.status {
        user_subscription.status = status;
    }
    
    if let Some(current_period_start) = updates.current_period_start {
        user_subscription.current_period_start = current_period_start;
    }
    
    if let Some(current_period_end) = updates.current_period_end {
        user_subscription.current_period_end = current_period_end;
    }

    // Update the user subscription in the database
    sqlx::query!(
        r#"
        UPDATE user_subscriptions 
        SET stripe_subscription_id = $1, stripe_product_id = $2, stripe_price_id = $3,
            stripe_plan_id = $4, stripe_plan_type = $5, credits = $6, cost = $7, status = $8,
            current_period_start = $9, current_period_end = $10, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = $11
        "#,
        user_subscription.stripe_subscription_id,
        user_subscription.stripe_product_id,
        user_subscription.stripe_price_id,
        user_subscription.stripe_plan_id,
        user_subscription.stripe_plan_type.as_str(),
        user_subscription.credits,
        user_subscription.cost,
        user_subscription.status.as_str(),
        user_subscription.current_period_start,
        user_subscription.current_period_end,
        user_id
    )
    .execute(pool)
    .await?;

    // Return the updated subscription
    Ok(user_subscription)
}
