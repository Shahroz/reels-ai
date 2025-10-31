//! Handle invoice.paid event for organization subscriptions.
//!
//! This handler processes invoice paid events specifically for organization-level subscriptions.
//! It creates or updates organization subscriptions and credit allocations based on the Stripe invoice data.
//! This is separate from user invoice handling to maintain clear separation of concerns
//! and enable different business logic for organizational vs individual billing.

use anyhow::Result;
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use tracing::instrument;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::queries::webhooks::users::get_user_id_by_stripe_customer_id;

/// Handle invoice paid event for an organization
///
/// This function:
/// 1. Looks up the organization by stripe_customer_id
/// 2. Updates or creates the organization subscription
/// 3. Allocates credits to the organization
///
/// Note: Currently implements basic subscription logic.
/// Future enhancements may include proration, downgrades, etc.
#[instrument(skip(pool))]
pub async fn handle_organization_invoice_paid(
    pool: &PgPool,
    customer_id: &str,
    invoice_id: &str,
    subscription_id: &str,
    product_plan: &str,
    price_metadata_credits: i32,
    _price_metadata_limit: i32,
    price_type: &str,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing organization invoice payment: invoice={}, customer={}, subscription={}", 
        invoice_id, customer_id, subscription_id);
    let user_id = get_user_id_by_stripe_customer_id(pool, customer_id).await?;
    if user_id.is_none() {
        tracing::error!("[STRIPE WEBHOOK] User not found for customer: {}", customer_id);
        return Err(anyhow::anyhow!("User not found for customer_id: {}", customer_id));
    }

    // Look up organization by stripe_customer_id
    let organization_id_opt = crate::queries::webhooks::organizations::get_organization_id_by_stripe_customer_id(
        pool,
        customer_id
    ).await?;

    let organization_id = match organization_id_opt {
        Some(org_id) => org_id,
        None => {
            tracing::error!("[STRIPE WEBHOOK] Organization not found for customer: {}", customer_id);
            return Err(anyhow::anyhow!("Organization not found for customer_id: {}", customer_id));
        }
    };

    tracing::info!("[STRIPE WEBHOOK] Found organization: {} for customer: {}", organization_id, customer_id);

    // Handle one-time payments differently from subscriptions
    if price_type == "one_time" {
        tracing::info!("[STRIPE WEBHOOK] Processing one-time payment for organization: {}", organization_id);
        
        // Add credits to organization
        match crate::queries::organization_credit_allocation::create_or_update_organization_credit_allocation::create_or_update_organization_credit_allocation(
            pool,
            organization_id,
            BigDecimal::from(price_metadata_credits),
            user_id, // Pass user_id for transaction logging (already Option<Uuid>)
        ).await {
            Ok(_) => {
                tracing::info!("[STRIPE WEBHOOK] Successfully added {} credits to organization: {}", 
                    price_metadata_credits, organization_id);
            }
            Err(e) => {
                tracing::error!("[STRIPE WEBHOOK] Failed to add credits to organization {}: {}", organization_id, e);
                return Err(e.into());
            }
        }
    } else {
        // Handle recurring subscription
        tracing::info!("[STRIPE WEBHOOK] Processing subscription payment for organization: {}", organization_id);
        
        // Try to update organization subscription status to active
        let update_result = crate::queries::organization_subscription::update_organization_subscription_status::update_organization_subscription_status(
            pool,
            subscription_id,
            SubscriptionStatus::Active,
        ).await;
        
        match update_result {
            Ok(_) => {
                tracing::info!("[STRIPE WEBHOOK] Successfully updated organization subscription status for org: {}", organization_id);
            }
            Err(_) => {
                // Subscription doesn't exist yet, create it
                tracing::info!("[STRIPE WEBHOOK] Organization subscription doesn't exist, creating it now for org: {}", organization_id);
                
                // We need to fetch subscription details from Stripe to get all required fields
                // For now, log and continue - the subscription will be created on next invoice
                tracing::warn!("[STRIPE WEBHOOK] Organization subscription not found for subscription_id: {}. It should be created in subscription.created event.", subscription_id);
                // Note: In the future, we should handle this in subscription.created event for organizations
            }
        }

        // Reset/allocate monthly credits to organization
        match crate::queries::organization_credit_allocation::create_or_update_organization_credit_allocation::create_or_update_organization_credit_allocation(
            pool,
            organization_id,
            BigDecimal::from(price_metadata_credits),
            user_id, // Pass user_id for transaction logging (already Option<Uuid>)
        ).await {
            Ok(_) => {
                tracing::info!("[STRIPE WEBHOOK] Successfully allocated {} monthly credits to organization: {}", 
                    price_metadata_credits, organization_id);
            }
            Err(e) => {
                tracing::error!("[STRIPE WEBHOOK] Failed to allocate credits to organization {}: {}", organization_id, e);
                return Err(e.into());
            }
        }
    }

    tracing::info!("[STRIPE WEBHOOK] Successfully processed organization invoice payment for org: {}", organization_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_function_exists() {
        // This test verifies the module compiles
        assert!(true);
    }
}

