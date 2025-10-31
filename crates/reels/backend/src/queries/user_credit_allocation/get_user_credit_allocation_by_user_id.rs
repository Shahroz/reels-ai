#![allow(clippy::disallowed_methods)]
//! Get user credit allocation by user ID.
//!
//! This function retrieves a user credit allocation record by user ID.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::user_credit_allocation::{UserCreditAllocation};
use crate::schemas::user_credit_allocation_schemas::DbUserCreditAllocation;

/// Get user credit allocation by user ID
#[instrument(skip(pool))]
pub async fn get_user_credit_allocation_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Option<UserCreditAllocation>, Error> {

    let result = sqlx::query_as!(
        DbUserCreditAllocation,
        r#"
        SELECT id, user_id, plan_type, daily_credits, plan_credits, credits_remaining,
               credit_limit, last_daily_credit_claimed_at, created_at, updated_at
        FROM user_credit_allocation
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|user_credit_allocation| user_credit_allocation.into_user_credit_allocation()))
}
