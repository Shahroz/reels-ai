#![allow(clippy::disallowed_methods)]
//! Create or update user credit allocation.
//!
//! This function creates or updates a user credit allocation record in the database.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;

/// Create or update user credit allocation
#[instrument(skip(pool))]
pub async fn create_or_update_user_credit_allocation(
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
    if user_credit_allocation.is_some() {
        log::info!("Updating user credit allocation for user: {}", user_id);
        
        // Checking for free plan
        let credits_remaining = if plan_type == StripePlanType::Free {
            // Reset the plan credits and credits remaining to the daily credits
            BigDecimal::from(plan_credits)
        } else {
            let current_credits = &user_credit_allocation.unwrap().credits_remaining + BigDecimal::from(plan_credits);
            // Checking if credits_remaining is greater than the credits_limit
            /*if current_credits > credit_limit {
                credit_limit
            } else {
                current_credits
            }*/
            current_credits
        };

        return crate::queries::user_credit_allocation::update_user_credit_allocation::update_user_credit_allocation(pool, user_id, plan_type, daily_credits, plan_credits, credits_remaining, credit_limit).await;
    } else {
        log::info!("User credit allocation does not exist, creating it");
        // If the user credit allocation does not exist, create it
        return crate::queries::user_credit_allocation::create_user_credit_allocation::create_user_credit_allocation(pool, user_id, plan_type, daily_credits, plan_credits, BigDecimal::from(plan_credits), credit_limit).await;
    }
}
