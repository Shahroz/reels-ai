//! Generic credits service for handling both user and organization credits.
//!
//! This service provides a unified interface for credit deduction that works
//! for both individual users and organizations. It routes to the appropriate
//! underlying query functions based on the context provided.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use actix_web::http::StatusCode;

use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;

/// Context for credit operations - either User or Organization
#[derive(Debug, Clone)]
pub enum CreditContext {
    User { user_id: Uuid },
    Organization { organization_id: Uuid, acting_user_id: Uuid },
}

/// Parameters for generic credit deduction
#[derive(Debug, Clone)]
pub struct DeductCreditsParams {
    pub context: CreditContext,
    pub credits_to_deduct: BigDecimal,
    pub action_source: String,
    pub action_type: String,
    pub entity_id: Option<Uuid>,
}

/// Result of a credit deduction operation
#[derive(Debug, Clone)]
pub struct CreditDeductionResult {
    pub previous_balance: BigDecimal,
    pub new_balance: BigDecimal,
    pub credits_deducted: BigDecimal,
}

/// Result of a credit availability check
#[derive(Debug, Clone)]
pub struct CreditAvailabilityError {
    pub message: String,
    pub status_code: StatusCode,
}

impl CreditAvailabilityError {
    pub fn new(message: String, status_code: StatusCode) -> Self {
        Self { message, status_code }
    }
}

/// Generic credit deduction function that works for both users and organizations
///
/// This function:
/// 1. Determines the context (user vs organization)
/// 2. Calls the appropriate deduction function
/// 3. Logs the transaction
/// 4. Returns the result
#[instrument(skip(pool))]
pub async fn deduct_credits(
    pool: &PgPool,
    params: DeductCreditsParams,
) -> Result<CreditDeductionResult, Error> {
    match params.context {
        CreditContext::User { user_id } => {
            deduct_user_credits_impl(pool, user_id, params).await
        }
        CreditContext::Organization { organization_id, acting_user_id } => {
            deduct_organization_credits_impl(
                pool,
                organization_id,
                acting_user_id,
                params
            ).await
        }
    }
}

async fn deduct_user_credits_impl(
    pool: &PgPool,
    user_id: Uuid,
    params: DeductCreditsParams,
) -> Result<CreditDeductionResult, Error> {
    // Clone the credits to deduct since we need it for transaction logging
    let credits_to_deduct = params.credits_to_deduct.clone();
    
    // Deduct credits and get both current and updated allocations
    let (current_allocation, updated_allocation) = 
        crate::queries::user_credit_allocation::deduct_user_credits::deduct_user_credits(
            pool,
            user_id,
            params.credits_to_deduct,
        ).await?;

    // Create transaction record
    let _transaction_result = create_credit_transaction(
        pool,
        CreateCreditTransactionParams {
            user_id,
            organization_id: None,
            credits_changed: -credits_to_deduct.clone(),
            previous_balance: current_allocation.credits_remaining.clone(),
            new_balance: updated_allocation.credits_remaining.clone(),
            action_source: params.action_source.clone(),
            action_type: params.action_type.clone(),
            entity_id: params.entity_id,
        },
    ).await?;

    log::info!(
        "User credit deduction: user_id={}, credits_deducted={}, previous={}, new={}, action={}:{}",
        user_id,
        credits_to_deduct,
        current_allocation.credits_remaining,
        updated_allocation.credits_remaining,
        params.action_source,
        params.action_type
    );

    Ok(CreditDeductionResult {
        previous_balance: current_allocation.credits_remaining,
        new_balance: updated_allocation.credits_remaining,
        credits_deducted: credits_to_deduct,
    })
}

/// Internal implementation for organization credit deduction
async fn deduct_organization_credits_impl(
    pool: &PgPool,
    organization_id: Uuid,
    acting_user_id: Uuid,
    params: DeductCreditsParams,
) -> Result<CreditDeductionResult, Error> {
    // Clone the credits to deduct since we need it for transaction logging
    let credits_to_deduct = params.credits_to_deduct.clone();
    
    // Deduct credits and get both current and updated allocations
    let (current_allocation, updated_allocation) = 
        crate::queries::organization_credit_allocation::deduct_organization_credits::deduct_organization_credits(
            pool,
            organization_id,
            params.credits_to_deduct.clone(),
        ).await?;

    // Create transaction record (logged under the acting user for audit purposes)
    let _transaction_result = create_credit_transaction(
        pool,
        CreateCreditTransactionParams {
            user_id: acting_user_id,
            organization_id: Some(organization_id),
            credits_changed: -credits_to_deduct.clone(),
            previous_balance: current_allocation.credits_remaining.clone(),
            new_balance: updated_allocation.credits_remaining.clone(),
            action_source: params.action_source.clone(),
            action_type: params.action_type.clone(),
            entity_id: params.entity_id,
        },
    ).await?;

    log::info!(
        "Organization credit deduction: org_id={}, acting_user_id={}, credits_deducted={}, previous={}, new={}, action={}:{}",
        organization_id,
        acting_user_id,
        credits_to_deduct,
        current_allocation.credits_remaining,
        updated_allocation.credits_remaining,
        params.action_source,
        params.action_type
    );

    Ok(CreditDeductionResult {
        previous_balance: BigDecimal::from(current_allocation.credits_remaining),
        new_balance: BigDecimal::from(updated_allocation.credits_remaining),
        credits_deducted: credits_to_deduct,
    })
}

/// Extract organization_id from request headers
/// 
/// This is a utility function for middleware and route handlers to extract
/// organization_id from HTTP headers. For route handlers that have access to
/// request payload, organization_id should be extracted from both header and payload
/// (with header taking precedence).
pub fn extract_organization_id_from_headers(req: &actix_web::HttpRequest) -> Option<Uuid> {
    if let Some(header_value) = req.headers().get("x-organization-id") {
        if let Ok(org_id_str) = header_value.to_str() {
            uuid::Uuid::parse_str(org_id_str).ok()
        } else {
            None
        }
    } else {
        None
    }
}

/// Check credit availability for a user or organization
///
/// This function checks if the user has sufficient credits to perform an operation.
/// It handles:
/// 1. Exempt users (old users)
/// 2. Organization credits (if organization_id is provided)
/// 3. User credits (default)
///
/// Returns Ok(()) if credits are available, or Err with error message and status code.
#[instrument(skip(pool))]
pub async fn check_credits_availability_by_user_or_organization(
    pool: &PgPool,
    user_id: Uuid,
    credits_to_consume: i32,
    organization_id: Option<Uuid>,
) -> Result<(), CreditAvailabilityError> {
    // Check if user is exempt from credit checks
    match crate::queries::user_credit_allocation::is_old_user_exempt_from_credit_checks::is_old_user_exempt_from_credit_checks(
        pool, user_id
    ).await {
        Ok(true) => {
            tracing::info!("User {} is an old user exempt from credit checks, allowing request", user_id);
            return Ok(());
        }
        Ok(false) => {
            // User is not exempt, check credits
        }
        Err(e) => {
            tracing::error!("Database error while checking old user status: {:?}", e);
            return Err(CreditAvailabilityError::new(
                format!("Failed to check user status: {e}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    // If organization_id is provided, check organization credits
    if let Some(org_id) = organization_id {
        // Verify user is a member of the organization
        match crate::queries::organizations::verify_organization_membership::verify_organization_membership(
            pool, user_id, org_id
        ).await {
            Ok(true) => {
                // Check organization credits
                match crate::queries::organization_credit_allocation::get_organization_credit_allocation_by_org_id::get_organization_credit_allocation_by_org_id(
                    pool, org_id
                ).await {
                    Ok(Some(org_allocation)) => {
                        if org_allocation.credits_remaining < bigdecimal::BigDecimal::from(credits_to_consume) {
                            return Err(CreditAvailabilityError::new(
                                format!(
                                    "Insufficient organization credits. This operation requires {} credits but the organization only has {} credits remaining.",
                                    credits_to_consume, org_allocation.credits_remaining
                                ),
                                StatusCode::PAYMENT_REQUIRED,
                            ));
                        }
                        // Credits are sufficient
                        return Ok(());
                    }
                    Ok(None) => {
                        return Err(CreditAvailabilityError::new(
                            "No credit allocation found for organization.".to_string(),
                            StatusCode::NOT_FOUND,
                        ));
                    }
                    Err(e) => {
                        log::error!("Failed to check organization credit availability for org {}: {}", org_id, e);
                        return Err(CreditAvailabilityError::new(
                            "Failed to verify credit availability. Please try again later.".to_string(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        ));
                    }
                }
            }
            Ok(false) => {
                return Err(CreditAvailabilityError::new(
                    "You are not a member of this organization.".to_string(),
                    StatusCode::FORBIDDEN,
                ));
            }
            Err(e) => {
                log::error!("Failed to verify organization membership: {}", e);
                return Err(CreditAvailabilityError::new(
                    "Failed to verify organization membership.".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    } else {
        // No organization_id provided, check user credits
        match crate::queries::user_credit_allocation::check_credits_availability(
            pool, user_id, credits_to_consume
        ).await {
            Ok(has_credits) => {
                if !has_credits {
                    return Err(CreditAvailabilityError::new(
                        format!(
                            "Insufficient credits. This operation requires {} credits but you don't have enough credits remaining.",
                            credits_to_consume
                        ),
                        StatusCode::PAYMENT_REQUIRED,
                    ));
                }
                // Credits are sufficient
                return Ok(());
            }
            Err(e) => {
                log::error!("Failed to check credit availability for user {}: {}", user_id, e);
                return Err(CreditAvailabilityError::new(
                    "Failed to verify credit availability. Please try again later.".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_context_creation() {
        let user_context = CreditContext::User {
            user_id: Uuid::new_v4(),
        };
        
        let org_context = CreditContext::Organization {
            organization_id: Uuid::new_v4(),
            acting_user_id: Uuid::new_v4(),
        };
        
        assert!(matches!(user_context, CreditContext::User { .. }));
        assert!(matches!(org_context, CreditContext::Organization { .. }));
    }
}

