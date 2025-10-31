//! Handler for customer.subscription.created Stripe webhook events.
//!
//! This handler processes subscription created events by extracting
//! subscription information, validating products, and creating user subscriptions.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::schemas::user_subscription_schemas::UserSubscriptionUpdates;
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use crate::queries::user_subscription::{create_user_subscription, get_user_subscription_by_stripe_price_id, update_user_subscription_by_user_id, cancel_all_subscriptions_except};

/// Handle subscription created event
#[instrument(skip(billing_service, pool, data))]
pub async fn handle_subscription_created_event(
    billing_service: &dyn BillingServiceTrait,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing customer.subscription.created event");

    if let Some(subscription) = data.get("object") {
        let customer_id = subscription
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let subscription_id = subscription
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing subscription ID"))?;

        let current_period_start = subscription
            .get("current_period_start")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let current_period_end = subscription
            .get("current_period_end")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        tracing::info!("[STRIPE WEBHOOK] Subscription: {subscription_id} for customer: {customer_id} (period: {current_period_start} to {current_period_end})");

        // Find user by Stripe customer ID
        let user_id = match crate::queries::webhooks::users::get_user_id_by_stripe_customer_id(pool, customer_id).await? {
            Some(id) => id,
            None => {
                tracing::warn!("[STRIPE WEBHOOK] No user found for customer: {customer_id}");
                return Ok(());
            }
        };

        // Extract the first price from subscription items
        let items = subscription
            .get("items")
            .and_then(|i| i.get("data"))
            .and_then(|d| d.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing subscription items"))?;

        if items.is_empty() {
            tracing::error!("[STRIPE WEBHOOK] No subscription items found for subscription: {subscription_id}");
            return Ok(());
        }

        let first_item = &items[0];

        // Extract price information
        let price = first_item
            .get("price")
            .ok_or_else(|| anyhow::anyhow!("Missing price in subscription item"))?;

        tracing::info!("[STRIPE WEBHOOK] Price: {price:?}");

        let price_id = price
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing price ID"))?;

        let price_type = price
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing price type"))?;

        let product_id = price
            .get("product")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing product ID in price"))?;

        // Extract plan ID from the subscription item
        let plan_id = first_item
            .get("plan")
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str()); // Returns Option<&str>

        tracing::info!(
            "[STRIPE WEBHOOK] Found plan: {:?} for product: {product_id}",
            plan_id
        );
        tracing::info!("[STRIPE WEBHOOK] Found price: {price_id} for product: {product_id}");

        // Log the plan object structure for debugging
        if let Some(plan_obj) = first_item.get("plan") {
            tracing::info!("[STRIPE WEBHOOK] Plan object: {plan_obj:?}");
        } else {
            tracing::info!("[STRIPE WEBHOOK] No plan object found, using price_id as plan_id");
        }

        // Fetch product details to check metadata
        let product = billing_service.get_product(product_id, false).await?;

        // Check if product_plan matches environment variable
        let stripe_metadata_product_type =
            std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default();
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

        tracing::info!("[STRIPE WEBHOOK] Product plan: {product_plan}");

        if product_plan == StripePlanType::Free.as_str() || price_type == "one_time" {
            tracing::info!("[STRIPE WEBHOOK] Product plan is free or price type is one_time, skipping subscription creation");
            return Ok(());
        }

        // Only proceed if product_type matches the environment variable
        if product_type == stripe_metadata_product_type {
            tracing::info!("[STRIPE WEBHOOK] Product plan matches, creating subscription and allocating credits");

            // Activate user subscription
            match billing_service
                .activate_user_subscription(pool, user_id, customer_id)
                .await
            {
                Ok(_) => {
                    tracing::info!("[STRIPE WEBHOOK] Successfully activated subscription for user: {user_id}");
                }
                Err(e) => {
                    tracing::error!("[STRIPE WEBHOOK] Failed to activate subscription: {e}");
                    return Err(e.into());
                }
            }

            // Convert plan type to CreditPlanType enum
            let credit_plan_type = StripePlanType::from_str(product_plan);
            tracing::info!("[STRIPE WEBHOOK] Credit plan type: {credit_plan_type}");

            if credit_plan_type == StripePlanType::Unknown {
                tracing::error!(
                    "[STRIPE WEBHOOK] Unknown product plan, skipping subscription creation"
                );
                return Ok(());
            }

            // Extract price amount and currency
            let unit_amount_cents = price
                .get("unit_amount")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            // Convert from cents to dollars for BigDecimal
            let cost = BigDecimal::from(unit_amount_cents) / BigDecimal::from(100);

            // Convert timestamps to DateTime
            let period_start = DateTime::from_timestamp(current_period_start, 0)
                .unwrap_or_else(|| Utc::now());
            let period_end = DateTime::from_timestamp(current_period_end, 0)
                .unwrap_or_else(|| Utc::now());

            let daily_credits = 2;
            let plan_credits = price
                .get("metadata")
                .and_then(|v| v.get("credits"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            let mut limit = price
                .get("metadata")
                .and_then(|v| v.get("limit"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            tracing::info!("[STRIPE WEBHOOK] Price metadata -> Plan credits: {plan_credits}, Limit: {limit}");
            if limit == 0 {
                if credit_plan_type == StripePlanType::Free {
                    limit = 30;
                } else {
                    if plan_credits == 0 {
                        limit = 30;
                    } else {
                        limit = (plan_credits as i32 * 2) as i64;
                    }
                }
            }

            // Determine credits based on plan type
            let (_daily_credits, _plan_credits, credits_remaining, _credit_limit) =
                match credit_plan_type {
                    StripePlanType::Free => (
                        daily_credits as i32,
                        plan_credits as i32,
                        plan_credits as i32,
                        limit as i32,
                    ),
                    StripePlanType::Pro => (
                        daily_credits as i32,
                        plan_credits as i32,
                        plan_credits as i32,
                        limit as i32,
                    ),
                    StripePlanType::Annual => (
                        daily_credits as i32,
                        plan_credits as i32,
                        plan_credits as i32,
                        limit as i32,
                    ),
                    StripePlanType::Unknown => (0, 0, 0, 0),
                };
            tracing::info!("[STRIPE WEBHOOK] Credits remaining: {credits_remaining}, Credit limit: {_credit_limit}");

            // Cancel all non free previous user subscription
            match cancel_all_subscriptions_except(pool, user_id, customer_id, subscription_id, billing_service).await {
                Ok(_subscriptions) => {
                    tracing::info!("[STRIPE WEBHOOK] Successfully canceled all previous subscriptions for user: {user_id}");
                }
                Err(e) => {
                    tracing::error!(
                        "[STRIPE WEBHOOK] Failed to cancel all previous subscriptions: {e}"
                    );
                }
            }

            // Fetch the created user subscription to verify it was stored correctly
            match get_user_subscription_by_stripe_price_id(pool, user_id, price_id)
                .await
            {
                Ok(Some(user_subscription)) => {
                    tracing::warn!(
                        "[STRIPE WEBHOOK] User subscription matched with price ID, updating it now"
                    );
                    let mut subscription_status = user_subscription.status.clone();
                    if subscription_status == SubscriptionStatus::Canceled || subscription_status == SubscriptionStatus::Expired {
                        tracing::warn!("[STRIPE WEBHOOK] Canceled or expired user subscription found for user: {user_id}, creating a new one now");
                        // Create user subscription
                        match create_user_subscription(
                            pool,
                            user_id,
                            subscription_id,
                            product_id,
                            price_id,
                            plan_id.unwrap_or(price_id), // Use price_id as fallback if plan_id is None
                            credit_plan_type,
                            credits_remaining,
                            cost,
                            SubscriptionStatus::from_str("unpaid"),
                            period_start,
                            period_end,
                        )
                        .await
                        {
                            Ok(_) => {
                                tracing::info!("[STRIPE WEBHOOK] Successfully created a new subscription for user: {user_id}");
                            }
                            Err(e) => {
                                tracing::error!("[STRIPE WEBHOOK] Failed to create a new user subscription for user: {user_id}: {e}");
                                return Err(e.into());
                            }
                        }
                    } else {
                     // Making sure the subscription status is set to unpaid if price id changed
                        subscription_status = SubscriptionStatus::Unpaid;
                        match update_user_subscription_by_user_id(
                            pool,
                            user_id,
                            Some(user_subscription),
                            UserSubscriptionUpdates {
                                stripe_subscription_id: Some(subscription_id.to_string()),
                                stripe_product_id: Some(product_id.to_string()),
                                stripe_price_id: Some(price_id.to_string()),
                                stripe_plan_id: plan_id.map(|s| s.to_string()),
                                stripe_plan_type: Some(credit_plan_type),
                                status: Some(subscription_status),
                                credits: Some(credits_remaining),
                                cost: Some(cost),
                                current_period_start: Some(period_start),
                                current_period_end: Some(period_end),
                            },
                        )
                        .await
                        {
                            Ok(_) => {
                                tracing::info!("[STRIPE WEBHOOK] Successfully updated user subscription for user: {user_id}");
                            }
                            Err(e) => {
                                tracing::error!(
                                    "[STRIPE WEBHOOK] Failed to update user subscription for user: {user_id}: {e}"
                                );
                                return Err(e.into());
                            }
                        }
                    }
                }
                Ok(None) => {
                    tracing::warn!("[STRIPE WEBHOOK] User subscription not found after creation for user: {user_id}, creating it now");
                    // Create user subscription
                    match create_user_subscription(
                        pool,
                        user_id,
                        subscription_id,
                        product_id,
                        price_id,
                        plan_id.unwrap_or(price_id), // Use price_id as fallback if plan_id is None
                        credit_plan_type,
                        credits_remaining,
                        cost,
                        SubscriptionStatus::from_str("unpaid"),
                        period_start,
                        period_end,
                    )
                    .await
                    {
                        Ok(_) => {
                            tracing::info!("[STRIPE WEBHOOK] Successfully created a new user subscription for user: {user_id}");
                        }
                        Err(e) => {
                            tracing::error!("[STRIPE WEBHOOK] Failed to create a new user subscription for user: {user_id}: {e}");
                            return Err(e.into());
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[STRIPE WEBHOOK] Failed to fetch user subscription after creation for user: {user_id}: {e}");
                    // Don't return error here as the subscription was created successfully
                }
            }
            tracing::info!("[STRIPE WEBHOOK] Successfully processed subscription creation for user: {user_id}");
        } else {
            tracing::info!("[STRIPE WEBHOOK] Product plan {product_plan} does not match expected {stripe_metadata_product_type}, skipping subscription creation");
        }
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No subscription object found in webhook data");
    }

    Ok(())
}
