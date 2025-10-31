#![allow(clippy::disallowed_methods)]
//! Create a new organization credit allocation.
//!
//! This function creates a new organization credit allocation record in the database.
//! Called when a new organization subscription is created.

use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::organization_credit_allocation::OrganizationCreditAllocation;

/// Create a new organization credit allocation
#[instrument(skip(pool))]
pub async fn create_organization_credit_allocation(
    pool: &PgPool,
    organization_id: Uuid,
    credits_remaining: BigDecimal,
) -> Result<OrganizationCreditAllocation, Error> {
    let result = sqlx::query_as!(
        OrganizationCreditAllocation,
        r#"
        INSERT INTO organization_credit_allocation (
            organization_id, credits_remaining, last_reset_date
        )
        VALUES ($1, $2, $3)
        RETURNING id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        "#,
        organization_id,
        credits_remaining,
        Utc::now()
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

