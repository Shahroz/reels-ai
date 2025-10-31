//! Check if user is eligible for credit rewards
//!
//! This function determines if a user is eligible for credit rewards based on:
//! 1. Product type must be "real_estate" (via STRIPE_METADATA_PRODUCT_TYPE env var)
//! 2. User must have a personal organization with 30 or fewer credits
//!
//! This encapsulates the business logic for credit reward eligibility that was
//! previously duplicated across multiple query functions.

use uuid::Uuid;
use sqlx::{PgPool, Result};
use crate::queries::organizations::get_user_personal_organization::get_user_personal_organization;
use bigdecimal::BigDecimal;

/// Check if user is eligible for credit rewards
/// 
/// Returns true if:
/// - STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
/// - User has a personal organization with less than or equal to 30 credits
/// 
/// Returns false otherwise.
#[tracing::instrument(skip(pool))]
pub async fn is_user_eligible_for_credit_rewards(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool> {
    // Verify STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
    let product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE")
        .unwrap_or_else(|_| "unknown".to_string());
    
    if product_type != "real_estate" {
        return Ok(false);
    }
    
    // Get user's personal organization
    let personal_org = match get_user_personal_organization(pool, user_id).await? {
        Some(org) => org,
        None => return Ok(false), // No personal organization means not eligible
    };
    
    // Check personal organization's credit allocation
    let org_credit_allocation = sqlx::query!(
        r#"
        SELECT credits_remaining
        FROM organization_credit_allocation
        WHERE organization_id = $1
        "#,
        personal_org.id
    )
    .fetch_optional(pool)
    .await?;
    
    // Only users with personal organizations having 30 or fewer credits should have tracking
    let is_eligible = org_credit_allocation.is_some() && 
        org_credit_allocation.as_ref().unwrap().credits_remaining <= BigDecimal::from(30);
    
    Ok(is_eligible)
}
