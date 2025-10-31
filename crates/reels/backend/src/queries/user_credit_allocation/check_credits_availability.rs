#![allow(clippy::disallowed_methods)]
//! Check if user has sufficient credits for the required amount.
//!
//! Returns true if user has enough credits, false otherwise.
//! For old users (created before 2024-12-01 with no subscription/credit records), always returns true.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;

/// Check if user has sufficient credits for the required amount
/// Returns true if user has enough credits, false otherwise
/// For old users (created before 2024-12-01 with no subscription/credit records), always returns true
#[instrument(skip(pool))]
pub async fn check_credits_availability(
    pool: &PgPool,
    user_id: Uuid,
    required_credits: i32,
) -> Result<bool, Error> {

    // Check if user has unlimited access grant
    if crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(pool, user_id).await? {
        log::info!("User {} has unlimited access grant, allowing operation", user_id);
        return Ok(true);
    }

    // Check if user has a subscription - if not, they might be a free user with credit allocation
    let statuses = [SubscriptionStatus::Trialing, SubscriptionStatus::Trial, SubscriptionStatus::Active];
    let user_subscription = crate::queries::users::get_user_by_statuses::get_user_by_statuses(pool, user_id, &statuses).await?;
    
    // If user has no subscription, check if they are a free user based on users.subscription_status
    if user_subscription.is_none() {
        let is_free_user = crate::queries::user_credit_allocation::is_user_free_user::is_user_free_user(pool, user_id).await?;
        if is_free_user {
            // This is a free user, check if they have a credit allocation
            let credit_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool, user_id).await?;
            if credit_allocation.is_some() {
                // This is a free user with credit allocation, proceed with credit check
                log::info!("User {} is a free user with credit allocation, proceeding with credit availability check", user_id);
            } else {
                return Err(Error::RowNotFound);
            }
        } else {
            return Err(Error::Protocol("User does not have an active subscription or credit allocation".into()));
        }
    } else {
        // User has subscription, check if it's active
        let user_subscription = user_subscription.unwrap();
        if user_subscription.status != SubscriptionStatus::Trialing.as_str() && user_subscription.status != SubscriptionStatus::Trial.as_str() && user_subscription.status != SubscriptionStatus::Active.as_str() {
            return Err(Error::Protocol("User does not have an active subscription".into()));
        }
    }

    // Validate input - ensure we're not checking for negative or zero credits
    if required_credits <= 0 {
        return Err(Error::Protocol("Required credits cannot be negative or zero".into()));
    }

    // Get the current credit allocation
    let current_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool, user_id).await?;

    let current_allocation = current_allocation
        .ok_or_else(|| Error::RowNotFound)?;

    // Check if user has enough credits
    Ok(current_allocation.credits_remaining >= BigDecimal::from(required_credits))
}
