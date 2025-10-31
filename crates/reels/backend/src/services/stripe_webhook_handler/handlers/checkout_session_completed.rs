//! Handler for checkout.session.completed Stripe webhook events.
//!
//! This handler processes checkout session completed events by extracting
//! customer information, handling one-time payments, and managing credit allocations.

use anyhow::Result;
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use tracing::instrument;

use crate::db::payment_completions::create_payment_completion;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use crate::queries::user_credit_allocation::create_or_update_user_credit_allocation_for_one_time_payment_with_transaction;

/// Handle checkout session completed event
#[instrument(skip(billing_service, pool, data))]
pub async fn handle_checkout_session_completed_event(
    billing_service: &dyn BillingServiceTrait,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing checkout.session.completed event");

    if let Some(session) = data.get("object") {
        let customer_id = session
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let session_id = session
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing session ID"))?;

        let subscription_id = session.get("subscription").and_then(|v| v.as_str());

        // Check metadata for customer_type to determine if this is an organization purchase
        let customer_type = session
            .get("metadata")
            .and_then(|metadata| metadata.get("customer_type"))
            .and_then(|ct| ct.as_str())
            .unwrap_or("user"); // Default to "user" for backwards compatibility

        // Extract price information from session metadata
        let price_id = session
            .get("metadata")
            .and_then(|metadata| metadata.get("price_id"))
            .and_then(|price_id| price_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing price_id in metadata"))?;

        tracing::info!("[STRIPE WEBHOOK] Found price_id in session metadata: {price_id}");

        tracing::info!("[STRIPE WEBHOOK] Customer type: {customer_type}");

        // DEBUG: Log the raw session data structure to understand what Stripe is sending
        tracing::info!("[STRIPE WEBHOOK] Raw checkout session data: {session:?}");
        tracing::info!(
            "[STRIPE WEBHOOK] total_details: {:?}",
            session.get("total_details")
        );
        tracing::info!("[STRIPE WEBHOOK] discounts: {:?}", session.get("discounts"));
        tracing::info!("[STRIPE WEBHOOK] metadata: {:?}", session.get("metadata"));
        tracing::info!(
            "[STRIPE WEBHOOK] amount_total: {:?}",
            session.get("amount_total")
        );
        tracing::info!(
            "[STRIPE WEBHOOK] amount_subtotal: {:?}",
            session.get("amount_subtotal")
        );

        // Additional debugging for promo code fields
        tracing::info!(
            "[STRIPE WEBHOOK] session.discounts[0].promotion_code: {:?}",
            session
                .get("discounts")
                .and_then(|d| d.as_array())
                .and_then(|arr| arr.first())
                .and_then(|discount| discount.get("promotion_code"))
        );
        tracing::info!("[STRIPE WEBHOOK] session.coupon: {:?}", session.get("coupon"));
        tracing::info!(
            "[STRIPE WEBHOOK] session.payment_intent: {:?}",
            session.get("payment_intent")
        );
        tracing::info!("[STRIPE WEBHOOK] session.mode: {:?}", session.get("mode"));
        tracing::info!(
            "[STRIPE WEBHOOK] session.payment_status: {:?}",
            session.get("payment_status")
        );

        // Extract promo code information from checkout session
        // First, check for promo code in the discounts array (this is where Stripe actually puts it)
        let promo_code = session
            .get("discounts")
            .and_then(|discounts| discounts.as_array())
            .and_then(|arr| arr.first())
            .and_then(|discount| discount.get("promotion_code"))
            .and_then(|promo| promo.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Fallback: check if there's a discount amount and try to extract coupon ID from discounts array
                let has_discount = session
                    .get("total_details")
                    .and_then(|total_details| total_details.get("amount_discount"))
                    .and_then(|amount_discount| amount_discount.as_i64())
                    .filter(|&amount| amount > 0) // amount_discount is already in cents
                    .is_some();

                if has_discount {
                    session
                        .get("discounts")
                        .and_then(|discounts| discounts.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|discount| discount.get("coupon"))
                        .and_then(|coupon| coupon.get("id"))
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            });

        // DEBUG: Log the promo code extraction process
        tracing::info!("[STRIPE WEBHOOK] Promo code extraction result: {promo_code:?}");
        if promo_code.is_none() {
            tracing::info!("[STRIPE WEBHOOK] No promo code found - checking individual components:");
            tracing::info!(
                "[STRIPE WEBHOOK] - discounts[0].promotion_code: {:?}",
                session
                    .get("discounts")
                    .and_then(|d| d.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|discount| discount.get("promotion_code"))
            );
            tracing::info!(
                "[STRIPE WEBHOOK] - total_details.amount_discount: {:?}",
                session
                    .get("total_details")
                    .and_then(|td| td.get("amount_discount"))
            );
            tracing::info!(
                "[STRIPE WEBHOOK] - discounts array: {:?}",
                session.get("discounts").and_then(|d| d.as_array())
            );
        }

        tracing::info!("[STRIPE WEBHOOK] Checkout session: {session_id} for customer: {customer_id} with subscription: {subscription_id:?}, promo_code: {promo_code:?}");
        
        // Fetch price details from Stripe
        let price = match billing_service.get_price(price_id).await {
            Ok(price) => price,
            Err(e) => {
                tracing::error!("[STRIPE WEBHOOK] Failed to fetch price details for {price_id}: {e}");
                return Err(e.into());
            }
        };
        let price_type = price.price_type
            .clone()
            .unwrap_or("unknown".to_string());
        
        tracing::info!("[STRIPE WEBHOOK] Price type for {price_id}: {price_type}");

        // Extract credits from price metadata
        let credits = price.metadata
            .get("credits")
            .and_then(|credits| credits.as_str())
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0);
        tracing::info!("[STRIPE WEBHOOK] Credits for {price_id}: {credits}");

        // Extract price metadata for credits and limits
        let price_metadata_credits = price.metadata
            .get("credits")
            .and_then(|credits| credits.as_str())
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0);

        // Extract payment method
        let payment_method = "card".to_string(); // Default for checkout sessions
        
        // Extract amount and currency from session
        let amount_total = session
            .get("amount_total")
            .and_then(|v| v.as_i64())
            .map(|v| v / 100)
            .unwrap_or(0) as i32;
        
        let currency = session
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("usd")
            .to_string();

        // Find user by Stripe customer ID
        let user_id = match crate::queries::webhooks::users::get_user_id_by_stripe_customer_id(pool, customer_id).await? {
            Some(id) => id,
            None => {
                tracing::warn!("[STRIPE WEBHOOK] No user found for customer: {customer_id}");
                return Ok(());
            }
        };
        tracing::info!("[STRIPE WEBHOOK] Found user: {user_id} for customer: {customer_id}");

        // Route to organization or user flow based on customer_type
        if customer_type == "organization" {
            // Organization purchase flow
            let organization_id = session
                .get("metadata")
                .and_then(|metadata| metadata.get("organization_id"))
                .and_then(|org_id| org_id.as_str())
                .and_then(|id_str| uuid::Uuid::parse_str(id_str).ok())
                .ok_or_else(|| anyhow::anyhow!("Missing or invalid organization_id in metadata"))?;

            tracing::info!("[STRIPE WEBHOOK] Processing organization purchase for org: {organization_id}");
            
            if price_type == "one_time" {
                tracing::info!("[STRIPE WEBHOOK] Adding {credits} credits to organization {organization_id}");

                // Add credits to organization
                match crate::queries::organization_credit_allocation::create_or_update_organization_credit_allocation::create_or_update_organization_credit_allocation(
                    pool,
                    organization_id,
                    BigDecimal::from(credits),
                    Some(user_id), // Pass user_id for transaction logging
                ).await {
                    Ok(_) => {
                        tracing::info!("[STRIPE WEBHOOK] Successfully added {credits} credits to organization: {organization_id}");
                    }
                    Err(e) => {
                        tracing::error!("[STRIPE WEBHOOK] Failed to add credits to organization: {e}");
                        return Err(e.into());
                    }
                }
            }
        } else {
            if price_type == "one_time" {
                // Get current user subscription to determine plan type, or default to Free
                let current_allocation = crate::queries::webhooks::payments::get_current_user_credit_allocation(pool, user_id).await?;
    
                let (product_plan, credit_limit) = current_allocation
                    .unwrap_or_else(|| ("free".to_string(), 0));
    
                tracing::info!("[STRIPE WEBHOOK] Using plan type for one-time payment: {product_plan}");
                
                // Create or update user credit allocation for one-time payment
                match create_or_update_user_credit_allocation_for_one_time_payment_with_transaction(
                    pool,
                    user_id,
                    StripePlanType::from_str(&product_plan),
                    2,
                    price_metadata_credits,
                    credit_limit,
                    None, // organization_id - TODO: Get from user context if available
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("[STRIPE WEBHOOK] Successfully created/updated credit allocation for one-time payment for user: {user_id}");
                    }
                    Err(e) => {
                        tracing::error!("[STRIPE WEBHOOK] Failed to create/update credit allocation for one-time payment: {e}");
                        return Err(e.into());
                    }
                }
            }
        }

        if price_type == "one_time" {
            // Check if payment completion already exists for this session
            let existing_payment = crate::queries::webhooks::payments::payment_completion_exists_by_session_id(pool, session_id).await?;
            
            if !existing_payment {
                tracing::info!("[STRIPE WEBHOOK] Creating payment completion for session: {session_id}");
                
                match create_payment_completion(
                    pool,
                    user_id,
                    session_id,
                    &payment_method,
                    amount_total,
                    &currency,
                    promo_code.as_deref(),
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("[STRIPE WEBHOOK] Stored payment completion for user: {user_id} via {payment_method} (checkout session)");
                    }
                    Err(e) => {
                        tracing::error!("[STRIPE WEBHOOK] Failed to store payment completion: {e}");
                    }
                }
            } else {
                tracing::info!("[STRIPE WEBHOOK] Payment completion already exists for session: {session_id}");
            }
        }

        // Note: We don't activate subscription here as it will be handled by
        // customer.subscription.created or invoice.payment_succeeded events
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No session object found in webhook data");
    }

    Ok(())
}
