//! Handler for invoice.paid Stripe webhook events.
//!
//! This handler processes invoice paid events by creating payment completions
//! and logging payment information for monitoring purposes.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

use crate::db::payment_completions::create_payment_completion;
use crate::queries::user_credit_allocation::{
    create_or_update_user_credit_allocation_for_one_time_payment_with_transaction,
    create_or_update_user_credit_allocation_with_transaction,
};
use crate::queries::user_subscription::{
    cancel_all_subscriptions_except, update_user_subscription_status,
};
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::services::billing::billing_service_trait::BillingServiceTrait;

/// Handle invoice paid event (preferred over invoice.payment_succeeded for idempotency)
#[instrument(skip(billing_service, pool, data))]
pub async fn handle_invoice_paid_event(
    billing_service: &dyn BillingServiceTrait,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing invoice.paid event");

    if let Some(invoice) = data.get("object") {
        let customer_id = invoice
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let invoice_id = invoice
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing invoice ID"))?;

        let paid = invoice
            .get("paid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Extract payment method using cleaner .and_then() chaining
        let payment_method = invoice
            .get("payment_intent")
            .and_then(|pi| pi.get("payment_method"))
            .and_then(|pm| pm.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("card")
            .to_string();

        // Extract amount and currency (convert from cents to dollars)
        let amount_cents = invoice
            .get("amount_paid")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let amount = amount_cents; // Keep as cents for payment completion storage

        let currency = invoice
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("usd")
            .to_string();

        tracing::info!(
            "[STRIPE WEBHOOK] Payment method: {payment_method}, amount: {amount} {currency}"
        );

        let billing_reason = invoice
            .get("billing_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!("[STRIPE WEBHOOK] Invoice: {invoice_id} paid: {paid} for customer: {customer_id} (reason: {billing_reason})");

        let subscription_id = invoice
            .get("subscription")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!("[STRIPE WEBHOOK] Subscription ID: {subscription_id}");

        // Getting first invoice item from invoice
        let first_invoice_item = invoice
            .get("lines")
            .and_then(|lines| lines.as_object())
            .and_then(|lines| lines.get("data"))
            .and_then(|data| data.as_array())
            .and_then(|data| data.first())
            .and_then(|item| item.as_object());

        // Extract plan object from first invoice item
        let plan = first_invoice_item
            .and_then(|item| item.get("plan"))
            .and_then(|plan| plan.as_object());

        // Extract product amount from plan (convert from cents to dollars)
        let plan_amount_cents = plan
            .and_then(|plan| plan.get("amount"))
            .and_then(|amount| amount.as_str())
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0);
        let plan_amount = plan_amount_cents; // Keep as cents for consistency

        // Extract price object from first invoice item
        let price = first_invoice_item
            .and_then(|item| item.get("price"))
            .and_then(|price| price.as_object());

        // Extract price metadata
        let price_metadata = price
            .and_then(|price| price.get("metadata"))
            .and_then(|metadata| metadata.as_object());

        // Extract price metadata credits
        let price_metadata_credits = price_metadata
            .and_then(|metadata| metadata.get("credits"))
            .and_then(|credits| credits.as_str())
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0);

        let price_metadata_limit = price_metadata
            .and_then(|metadata| metadata.get("limit"))
            .and_then(|limit| limit.as_str())
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0);

        let product_id = price
            .and_then(|price| price.get("product"))
            .and_then(|product| product.as_str())
            .unwrap_or("unknown");

        let price_type = price
            .and_then(|price| price.get("type"))
            .and_then(|product| product.as_str())
            .unwrap_or("unknown");

        tracing::info!("[STRIPE WEBHOOK] Product ID: {product_id}, Price type: {price_type}");

        // Fetch product details to check metadata
        let product = billing_service.get_product(product_id, false).await?;

        // Check if product_plan matches environment variable
        let product_plan = product
            .metadata
            .get("product_plan")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let product_type = product
            .metadata
            .get("product_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!(
            "[STRIPE WEBHOOK] Product plan: {product_plan}, Product type: {product_type}"
        );

        // Get promo code from webhook_events table (stored during checkout.session.completed)
        let promo_code = {
            if !subscription_id.is_empty() {
                tracing::info!("[STRIPE WEBHOOK] Looking for promo code in webhook_events for subscription: {subscription_id}");

                // Try to get promo code directly from webhook_events table
                // We now store the checkout session ID in webhook_events during checkout.session.completed
                match crate::queries::webhooks::webhook_events::get_latest_promo_code_from_webhook_events(pool).await {
                    Ok(Some(stored_code)) => {
                        tracing::info!("[STRIPE WEBHOOK] Found promo code in webhook_events: {stored_code}");
                        Some(stored_code)
                    }
                    Ok(None) => {
                        tracing::info!("[STRIPE WEBHOOK] No promo code found in webhook_events");
                        None
                    }
                    Err(e) => {
                        tracing::warn!("[STRIPE WEBHOOK] Error retrieving promo code from webhook_events: {e}");
                        None
                    }
                }
            } else {
                tracing::info!("[STRIPE WEBHOOK] No subscription ID found in invoice, falling back to invoice discounts");
                // Fall back to invoice discounts if no subscription
                let fallback_code = invoice
                    .get("discounts")
                    .and_then(|discounts| discounts.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|discount| discount.get("coupon"))
                    .and_then(|coupon| coupon.get("id"))
                    .and_then(|id| id.as_str())
                    .map(|s| s.to_string());
                tracing::info!("[STRIPE WEBHOOK] Fallback promo code from invoice discounts: {fallback_code:?}");
                fallback_code
            }
        };

        if let Some(ref code) = promo_code {
            tracing::info!("[STRIPE WEBHOOK] Promo code applied: {code} for invoice: {invoice_id}");
        }

        // DEBUG: Log invoice data structure to understand what Stripe is sending
        tracing::info!("[STRIPE WEBHOOK] Raw invoice data: {invoice:?}");
        tracing::info!("[STRIPE WEBHOOK] invoice.subscription: {subscription_id}");
        tracing::info!(
            "[STRIPE WEBHOOK] invoice.discounts: {:?}",
            invoice.get("discounts")
        );

        if product_plan != "unknown" && product_type != "unknown" {
            if product_plan == StripePlanType::Free.as_str() {
                // Check if payment completion already exists for this invoice
                let existing_payment =
                    crate::queries::webhooks::payments::payment_completion_exists_by_session_id(
                        pool, invoice_id,
                    )
                    .await?;

                if !existing_payment {
                    tracing::info!(
                        "[STRIPE WEBHOOK] Product plan is free, adding payment completion"
                    );
                    // Fetch current user state to ensure we're working with latest data
                    let user_id =
                        match crate::queries::webhooks::users::get_user_id_by_stripe_customer_id(
                            pool,
                            customer_id,
                        )
                        .await?
                        {
                            Some(id) => id,
                            None => {
                                tracing::warn!(
                                    "[STRIPE WEBHOOK] No user found for customer: {customer_id}"
                                );
                                return Ok(());
                            }
                        };

                    // Create payment completion with idempotency
                    match create_payment_completion(
                        pool,
                        user_id,
                        invoice_id, // Use invoice_id as session_id for invoice payments
                        &payment_method,
                        amount,
                        &currency,
                        promo_code.as_deref(),
                    )
                    .await
                    {
                        Ok(_) => {
                            tracing::info!("[STRIPE WEBHOOK] Stored payment completion for user: {user_id} via {payment_method} (invoice)");
                        }
                        Err(e) => {
                            tracing::error!(
                                "[STRIPE WEBHOOK] Failed to store payment completion: {e}"
                            );
                        }
                    }
                }
            }

            if product_plan == StripePlanType::Free.as_str() {
                tracing::info!(
                    "[STRIPE WEBHOOK] Product plan is free, skipping invoice processing"
                );
                return Ok(());
            } else if product_type
                != std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default()
            {
                tracing::info!(
                    "[STRIPE WEBHOOK] Product type is not {}, skipping invoice processing",
                    std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default()
                );
                return Ok(());
            }

            // Only process if invoice is paid
            if paid {
                // Fetch customer from Stripe to get metadata and determine customer_type
                tracing::info!(
                    "[STRIPE WEBHOOK] Fetching customer {} from Stripe to determine type",
                    customer_id
                );
                let customer = match billing_service.get_customer(customer_id).await {
                    Ok(cust) => cust,
                    Err(e) => {
                        tracing::error!(
                            "[STRIPE WEBHOOK] Failed to fetch customer {}: {}",
                            customer_id,
                            e
                        );
                        return Err(anyhow::anyhow!(
                            "Failed to fetch customer from Stripe: {}",
                            e
                        ));
                    }
                };

                // Extract customer_type from metadata
                let customer_type = customer
                    .metadata
                    .get("customer_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user"); // Default to "user" for backwards compatibility

                tracing::info!(
                    "[STRIPE WEBHOOK] Customer {} has type: {}",
                    customer_id,
                    customer_type
                );

                // Route based on customer_type
                if customer_type == "user" {
                    // Existing user flow
                    tracing::info!(
                        "[STRIPE WEBHOOK] Processing invoice for user customer: {}",
                        customer_id
                    );

                    // Fetch current user state to ensure we're working with latest data
                    let user_result = crate::queries::webhooks::users::get_user_id_and_subscription_status_by_stripe_customer_id(pool, customer_id).await?;

                    match user_result {
                        Some((user_id, current_status)) => {
                            let current_status = current_status.as_deref().unwrap_or("unknown");

                            tracing::info!("[STRIPE WEBHOOK] Found user: {user_id} with current subscription status: {current_status}");

                            if price_type == "one_time" {
                                // Create or update user credit allocation for one-time payment
                                match create_or_update_user_credit_allocation_for_one_time_payment_with_transaction(
                                pool,
                                user_id,
                                StripePlanType::from_str(product_plan),
                                2,
                                price_metadata_credits,
                                price_metadata_limit,
                                None, // organization_id - TODO: Get from user context if available
                            )
                            .await
                            {
                                Ok(_) => {
                                    tracing::info!("[STRIPE WEBHOOK] Successfully created/updated credit allocation for user: {user_id}");
                                }
                                Err(e) => {
                                    tracing::error!("[STRIPE WEBHOOK] Failed to create/update credit allocation: {e}");
                                    return Err(e.into());
                                }
                            }
                            } else {
                                // Cancel all subscriptions for customer except the one with id subscription_id
                                match cancel_all_subscriptions_except(
                                    pool,
                                    user_id,
                                    customer_id,
                                    subscription_id,
                                    billing_service,
                                )
                                .await
                                {
                                    Ok(subscriptions) => {
                                        tracing::info!("[STRIPE WEBHOOK] Successfully canceled all subscriptions except the one with id {subscription_id} for user: {user_id}");
                                        for subscription in subscriptions {
                                            tracing::info!("[STRIPE WEBHOOK] Canceled subscription: {subscription}");
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("[STRIPE WEBHOOK] Failed to cancel all subscriptions except the one with id {subscription_id}: {e}");
                                    }
                                }

                                // Update user subscription status to active
                                match update_user_subscription_status(
                                    pool,
                                    subscription_id,
                                    SubscriptionStatus::Active,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        tracing::info!("[STRIPE WEBHOOK] Successfully updated user subscription status for user: {user_id}");
                                    }
                                    Err(e) => {
                                        tracing::error!("[STRIPE WEBHOOK] Failed to update user subscription status: {e}");
                                    }
                                }

                                // Create or update user credit allocation
                                match create_or_update_user_credit_allocation_with_transaction(
                                    pool,
                                    user_id,
                                    StripePlanType::from_str(product_plan),
                                    2,
                                    price_metadata_credits,
                                    price_metadata_limit,
                                    None, // organization_id - TODO: Get from user context if available
                                )
                                .await
                                {
                                    Ok(_) => {
                                        tracing::info!("[STRIPE WEBHOOK] Successfully created/updated credit allocation for user: {user_id}");
                                    }
                                    Err(e) => {
                                        tracing::error!("[STRIPE WEBHOOK] Failed to create/update credit allocation: {e}");
                                    }
                                }

                                // Activate user subscription
                                tracing::info!("[STRIPE WEBHOOK] Attempting to activate subscription for user: {user_id} with customer: {customer_id}");
                                let active_status = SubscriptionStatus::Active.as_str();
                                match crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, active_status).await {
                                Ok(_) => {
                                    tracing::info!("[STRIPE WEBHOOK] Successfully activated subscription for user: {user_id} from invoice: {invoice_id}");

                                    tracing::info!("[STRIPE WEBHOOK] User {} subscription status after activation: {:?}", 
                                            user_id, active_status);

                                    // Track subscription activation analytics (non-blocking)
                                    crate::routes::stripe::analytics_helper::track_subscription_activation(
                                        pool,
                                        user_id,
                                        current_status, // Previous status - typically trial when first invoice is paid
                                        active_status, // New status after payment
                                    ).await;
                                }
                                Err(e) => {
                                    tracing::error!("[STRIPE WEBHOOK] Failed to activate subscription for user: {user_id}: {e}");
                                }
                            }
                            }

                            // Check if payment completion already exists for this invoice
                            let existing_payment = crate::queries::webhooks::payments::payment_completion_exists_by_session_id(pool, invoice_id).await?;

                            if existing_payment {
                                tracing::info!("[STRIPE WEBHOOK] Payment completion already exists for invoice: {invoice_id}, skipping");
                                return Ok(());
                            }

                            // Create payment completion with idempotency
                            match create_payment_completion(
                                pool,
                                user_id,
                                invoice_id, // Use invoice_id as session_id for invoice payments
                                &payment_method,
                                amount,
                                &currency,
                                promo_code.as_deref(),
                            )
                            .await
                            {
                                Ok(_) => {
                                    tracing::info!("[STRIPE WEBHOOK] Stored payment completion for user: {user_id} via {payment_method} (invoice)");
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "[STRIPE WEBHOOK] Failed to store payment completion: {e}"
                                    );
                                }
                            }
                        }
                        None => {
                            tracing::error!(
                                "[STRIPE WEBHOOK] User not found for customer ID: {customer_id}"
                            );
                        }
                    }
                } else if customer_type == "organization" {
                    // Organization flow
                    tracing::info!(
                        "[STRIPE WEBHOOK] Processing invoice for organization customer: {}",
                        customer_id
                    );

                    // Call organization invoice handler
                    match crate::services::stripe_webhook_handler::handlers::handle_organization_invoice_paid::handle_organization_invoice_paid(
                        pool,
                        customer_id,
                        invoice_id,
                        subscription_id,
                        product_plan,
                        price_metadata_credits,
                        price_metadata_limit,
                        price_type,
                    ).await {
                        Ok(_) => {
                            tracing::info!("[STRIPE WEBHOOK] Successfully processed organization invoice for customer: {}", customer_id);
                        }
                        Err(e) => {
                            tracing::error!("[STRIPE WEBHOOK] Failed to process organization invoice: {}", e);
                            return Err(e);
                        }
                    }
                } else {
                    tracing::warn!(
                        "[STRIPE WEBHOOK] Unknown customer_type '{}' for customer: {}",
                        customer_type,
                        customer_id
                    );
                }
            } else {
                tracing::info!(
                    "[STRIPE WEBHOOK] Invoice status is not 'paid', current status: {paid}"
                );
            }
        } else {
            tracing::info!("[STRIPE WEBHOOK] Product plan or product type is unknown, skipping invoice processing");
        }
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No invoice object found in webhook data");
    }
    Ok(())
}
