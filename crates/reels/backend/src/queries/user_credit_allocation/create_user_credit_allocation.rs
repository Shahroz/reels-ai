#![allow(clippy::disallowed_methods)]
//! Create user credit allocation.
//!
//! This function creates a new user credit allocation record in the database.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::schemas::user_credit_allocation_schemas::DbUserCreditAllocation;

/// Create user credit allocation
#[instrument(skip(pool))]
pub async fn create_user_credit_allocation(
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
        INSERT INTO user_credit_allocation (user_id, plan_type, daily_credits, plan_credits, credits_remaining, credit_limit, last_daily_credit_claimed_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, plan_type, daily_credits, plan_credits, credits_remaining,
                  credit_limit, last_daily_credit_claimed_at, created_at, updated_at
        "#,
        user_id,
        plan_type.as_str(),
        daily_credits,
        plan_credits,
        credits_remaining,
        credit_limit,
        if daily_credits > 0 { Some(Utc::now()) } else { None::<DateTime<Utc>> } // Set last_daily_credit_claimed_at to current time if daily_credits > 0, otherwise NULL
    )
    .fetch_one(pool)
    .await?;

    Ok(result.into_user_credit_allocation())
}
