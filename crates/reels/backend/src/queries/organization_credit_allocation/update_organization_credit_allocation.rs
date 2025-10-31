#![allow(clippy::disallowed_methods)]
//! Update organization credit allocation.
//!
//! This function updates the credits remaining for an organization.
//! Used for both refilling credits and direct updates.

use bigdecimal::BigDecimal;
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::organization_credit_allocation::OrganizationCreditAllocation;

/// Update organization credit allocation
#[instrument(skip(pool))]
pub async fn update_organization_credit_allocation(
    pool: &PgPool,
    organization_id: Uuid,
    credits_remaining: BigDecimal
) -> Result<OrganizationCreditAllocation, Error> {
    let result = sqlx::query_as!(
        OrganizationCreditAllocation,
        r#"
        UPDATE organization_credit_allocation
        SET credits_remaining = $1, updated_at = CURRENT_TIMESTAMP
        WHERE organization_id = $2
        RETURNING id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        "#,
        credits_remaining,
        organization_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

