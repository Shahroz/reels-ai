#![allow(clippy::disallowed_methods)]
//! Update user credit allocation.
//!
//! This function updates a user credit allocation record in the database.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::schemas::user_credit_allocation_schemas::DbUserCreditAllocation;

/// Update user credit allocation
#[instrument(skip(pool))]
pub async fn update_user_credit_allocation(
    pool: &PgPool,
    user_id: Uuid,
    plan_type: StripePlanType,
    daily_credits: i32,
    plan_credits: i32,
    credits_remaining: BigDecimal,
    credit_limit: i32,
) -> Result<UserCreditAllocation, Error> {

    let result = sqlx::query_as!(
        DbUserCreditAllocation,
        r#"
        UPDATE user_credit_allocation 
        SET plan_type = $1, daily_credits = $2, plan_credits = $3, 
            credits_remaining = $4, credit_limit = $5, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = $6
        RETURNING id, user_id, plan_type, daily_credits, plan_credits, credits_remaining,
                  credit_limit, last_daily_credit_claimed_at, created_at, updated_at
        "#,
        plan_type.as_str(),
        daily_credits,
        plan_credits,
        credits_remaining,
        credit_limit,
        user_id
    )
    .fetch_one(pool)
    .await?;

    let allocation = UserCreditAllocation {
        id: result.id,
        user_id: result.user_id,
        plan_type: StripePlanType::from_str(&result.plan_type),
        daily_credits: result.daily_credits,
        plan_credits: result.plan_credits,
        credits_remaining: result.credits_remaining,
        credit_limit: result.credit_limit,
        last_daily_credit_claimed_at: result.last_daily_credit_claimed_at,
        created_at: result.created_at,
        updated_at: result.updated_at,
    };
    
    Ok(allocation)
}
