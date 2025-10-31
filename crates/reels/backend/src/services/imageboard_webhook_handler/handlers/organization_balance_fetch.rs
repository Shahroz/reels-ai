use uuid::Uuid;
use sqlx::PgPool;
use bigdecimal::BigDecimal;

use crate::app_constants::credits_constants::CREDITS_TO_CENTS_RATIO;
use crate::schemas::imageboard_schemas::{OrganizationCreditsAllocationLiteResponse, WebhookErrorResponse};

pub async fn handle_org_balance_fetch(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<OrganizationCreditsAllocationLiteResponse, WebhookErrorResponse> {
    let _organization = crate::queries::organizations::find_organization_by_id(pool, organization_id)
        .await
        .map_err(|_| WebhookErrorResponse{
            error: "Internal server error".to_string(),
            code: "INTERNAL_SERVER_ERROR".to_string(),
            details: Some("Failed to fetch organization".to_string()),
        })?
        .ok_or_else(|| WebhookErrorResponse{
            error: "Organization not found".to_string(),
            code: "ORGANIZATION_NOT_FOUND".to_string(),
            details: None,
        })?;

    let org_credit_allocation = crate::queries::organization_credit_allocation::get_organization_credit_allocation_by_org_id::get_organization_credit_allocation_by_org_id(pool, organization_id)
        .await
        .map_err(|_| WebhookErrorResponse{
            error: "Internal server error".to_string(),
            code: "INTERNAL_SERVER_ERROR".to_string(),
            details: Some("Failed to retrieve organization credit allocation".to_string()),
        })?
        .ok_or_else(|| WebhookErrorResponse{
            error: "Organization credit allocation not found".to_string(),
            code: "ORG_CREDIT_ALLOCATION_NOT_FOUND".to_string(),
            details: None,
        })?;

    let amount_cents = &org_credit_allocation.credits_remaining * BigDecimal::from(CREDITS_TO_CENTS_RATIO);
    let balance_cents = amount_cents.to_string().parse::<i32>().unwrap_or(0);
    Ok(OrganizationCreditsAllocationLiteResponse{
        balance_cents,
    })
}


