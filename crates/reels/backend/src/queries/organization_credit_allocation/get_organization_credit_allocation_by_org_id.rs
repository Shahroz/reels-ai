#![allow(clippy::disallowed_methods)]
//! Get organization credit allocation by organization ID.
//!
//! This function retrieves an organization credit allocation record by organization ID.
//! Returns None if no allocation exists for the organization.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::organization_credit_allocation::OrganizationCreditAllocation;

/// Get organization credit allocation by organization ID
#[instrument(skip(pool))]
pub async fn get_organization_credit_allocation_by_org_id(
    pool: &PgPool,
    organization_id: Uuid
) -> Result<Option<OrganizationCreditAllocation>, Error> {
    let result = sqlx::query_as!(
        OrganizationCreditAllocation,
        r#"
        SELECT id, organization_id, credits_remaining, last_reset_date, created_at, updated_at
        FROM organization_credit_allocation
        WHERE organization_id = $1
        "#,
        organization_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

