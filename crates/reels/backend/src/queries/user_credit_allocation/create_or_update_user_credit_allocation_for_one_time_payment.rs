#![allow(clippy::disallowed_methods)]
//! Create or update user credit allocation for one-time payment.
//!
//! This function creates or updates a user credit allocation record for one-time payments.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;

/// Create or update user credit allocation for one-time payment
#[instrument(skip(pool))]
pub async fn create_or_update_user_credit_allocation_for_one_time_payment(
    pool: &PgPool,
    user_id: Uuid,
    plan_type: StripePlanType,
    daily_credits: i32,
    plan_credits: i32,
    credit_limit: i32,
) -> Result<UserCreditAllocation, Error> {
    // Fetch the user credit allocation
    let user_credit_allocation = crate::queries::user_credit_allocation::get_user_credit_allocation_by_user_id::get_user_credit_allocation_by_user_id(pool, user_id).await?;

    // If the user credit allocation exists, update it
    if let Some(allocation) = user_credit_allocation {
        log::info!("Updating user credit allocation for user: {}", user_id);
        let total_credits = &allocation.credits_remaining + BigDecimal::from(plan_credits);

        return crate::queries::user_credit_allocation::update_user_credit_allocation::update_user_credit_allocation(pool, user_id, allocation.plan_type, daily_credits, allocation.plan_credits, total_credits, allocation.credit_limit).await;
    } else {
        log::info!("User credit allocation does not exist, creating it");
        // If the user credit allocation does not exist, create it
        return crate::queries::user_credit_allocation::create_user_credit_allocation::create_user_credit_allocation(pool, user_id, plan_type, daily_credits, plan_credits, BigDecimal::from(plan_credits), credit_limit).await;
    }
}
