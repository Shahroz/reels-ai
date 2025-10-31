//! Handler for customer.subscription.updated Stripe webhook events.
//!
//! This handler processes subscription updated events by extracting
//! subscription status changes and updating user subscription status accordingly.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::queries::user_credit_allocation::update_user_credit_allocation;
use crate::queries::user_subscription::{update_user_subscription_status, cancel_all_subscriptions_except};
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use bigdecimal::BigDecimal;

/// Handle subscription updated event
#[instrument(skip(billing_service, pool, data))]
pub async fn handle_subscription_updated_event(
    billing_service: &dyn BillingServiceTrait,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing customer.subscription.updated event");

    if let Some(subscription) = data.get("object") {
        let customer_id = subscription
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let subscription_id = subscription
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing subscription ID"))?;

        let status = subscription
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let cancel_at_period_end = subscription
            .get("cancel_at_period_end")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let current_period_end = subscription
            .get("current_period_end")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        tracing::info!("[STRIPE WEBHOOK] Subscription: {subscription_id} status: {status} for customer: {customer_id} (cancel_at_period_end: {cancel_at_period_end}, period_end: {current_period_end})");

        // Extract the first price from subscription items to get product information
        let items = subscription
            .get("items")
            .and_then(|i| i.get("data"))
            .and_then(|d| d.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing subscription items"))?;

        if items.is_empty() {
            tracing::error!(
                "[STRIPE WEBHOOK] No subscription items found for subscription: {subscription_id}"
            );
            return Ok(());
        }

        let first_item = &items[0];

        // Extract price information
        let price = first_item
            .get("price")
            .ok_or_else(|| anyhow::anyhow!("Missing price in subscription item"))?;

        let product_id = price
            .get("product")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing product ID in price"))?;

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

        tracing::info!("[STRIPE WEBHOOK] Product plan: {product_plan}, expected: {stripe_metadata_product_type}");

        if product_plan == StripePlanType::Free.as_str() {
            tracing::info!("[STRIPE WEBHOOK] Product plan is free, skipping subscription update");
            return Ok(());
        }

        // Only proceed if product_type matches the environment variable
        if product_type != stripe_metadata_product_type {
            tracing::info!(
                "[STRIPE WEBHOOK] Product type does not match, skipping subscription update"
            );
            return Ok(());
        }

        tracing::info!("[STRIPE WEBHOOK] Product type matches, proceeding with subscription update");

        // Find user by Stripe customer ID
        let user_result = sqlx::query!(
            "SELECT id, email FROM users WHERE stripe_customer_id = $1",
            customer_id
        )
        .fetch_one(pool)
        .await;

        match user_result {
            Ok(user_record) => {
                let user_id = user_record.id;

                // Handle different subscription states
                match status {
                    "active" => {
                        if cancel_at_period_end {
                            tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} will be canceled at period end for user: {user_id}");
                            // Update user to canceled (access remains until period end)
                            crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "canceled").await?;

                            // Update user subscription status to canceled in user_subscriptions table
                            update_user_subscription_status(
                                pool,
                                subscription_id,
                                SubscriptionStatus::Canceled,
                            )
                            .await?;

                            tracing::info!("[STRIPE WEBHOOK] User {user_id} subscription marked as canceled (access until period end)");

                            // Track subscription cancellation scheduled analytics (non-blocking)
                            crate::routes::stripe::analytics_helper::track_subscription_cancellation_scheduled(
                                pool,
                                user_id,
                                "canceled", // Status is now canceled (access until period end)
                                current_period_end,
                            ).await;
                        } else {
                            tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is active for user: {user_id}");

                            // Update user subscription status to active in user_subscriptions table
                            update_user_subscription_status(
                                pool,
                                subscription_id,
                                SubscriptionStatus::Active,
                            )
                            .await?;

                            // Ensure user is marked as active
                            crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "active").await?;
                            tracing::info!("[STRIPE WEBHOOK] User {user_id} subscription status updated to active");

                            // Track subscription activation analytics (non-blocking)
                            crate::routes::stripe::analytics_helper::track_subscription_activation(
                                pool, user_id, "trial", "active",
                            )
                            .await;
                        }
                    }
                    "canceled" => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is canceled for user: {user_id}");
                        // Revoke access by setting subscription status to canceled
                        crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "canceled").await?;
                        tracing::info!("[STRIPE WEBHOOK] User {user_id} access revoked due to canceled subscription");

                        // Cancel all non free previous user subscription
                        match cancel_all_subscriptions_except(pool, user_id, customer_id, subscription_id, billing_service)
                            .await
                        {
                            Ok(_subscriptions) => {
                                tracing::info!("[STRIPE WEBHOOK] Successfully canceled all non-free subscriptions for user: {user_id}");
                            }
                            Err(e) => {
                                tracing::error!("[STRIPE WEBHOOK] Failed to cancel all non-free subscriptions: {e}");
                            }
                        }
                        // Track subscription canceled analytics (non-blocking) - immediate cancellation
                        crate::routes::stripe::analytics_helper::track_subscription_canceled(
                            pool, user_id, "active", // Previous status was likely active
                            false,    // This is an immediate cancellation, not scheduled
                        )
                        .await;

                        // Update user credit allocation
                        match update_user_credit_allocation(
                            pool,
                            user_id,
                            StripePlanType::Free,
                            2,  // daily_credits
                            0,  // plan_credits
                            BigDecimal::from(30), // credits_remaining
                            30, // credit_limit
                        )
                        .await
                        {
                            Ok(_) => {
                                tracing::info!("[STRIPE WEBHOOK] Successfully updated user credit allocation for user: {user_id}");
                            }
                            Err(e) => {
                                tracing::error!(
                                    "[STRIPE WEBHOOK] Failed to update user credit allocation: {e}"
                                );
                            }
                        }
                    }
                    "past_due" => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is past due for user: {user_id}");
                        // Move user to expired status (past due = subscription expired but not canceled)
                        crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "expired").await?;
                        tracing::info!(
                            "[STRIPE WEBHOOK] User {user_id} moved to grace period (past_due)"
                        );

                        // Update user subscription status to past due in user_subscriptions table
                        update_user_subscription_status(
                            pool,
                            subscription_id,
                            SubscriptionStatus::PastDue,
                        )
                        .await?;
                        tracing::info!("[STRIPE WEBHOOK] User {user_id} subscription marked as expired due to past due status");
                    }
                    "incomplete_expired" => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is incomplete expired for user: {user_id}");
                        // Move user to incomplete expired status
                        crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "incomplete_expired").await?;
                        tracing::info!(
                            "[STRIPE WEBHOOK] User {user_id} moved to incomplete expired status"
                        );

                        // Update user subscription status to incomplete expired in user_subscriptions table
                        update_user_subscription_status(
                            pool,
                            subscription_id,
                            SubscriptionStatus::IncompleteExpired,
                        )
                        .await?;
                    }
                    "incomplete" => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is incomplete for user: {user_id}");
                        // Move user to incomplete status
                        crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "incomplete").await?;
                        tracing::info!("[STRIPE WEBHOOK] User {user_id} moved to incomplete status");

                        // Update user subscription status to incomplete in user_subscriptions table
                        update_user_subscription_status(
                            pool,
                            subscription_id,
                            SubscriptionStatus::Incomplete,
                        )
                        .await?;
                    }
                    "unpaid" => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is unpaid for user: {user_id}");
                        // Move user to expired status (unpaid = subscription expired due to failed payment)
                        crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "expired").await?;
                        tracing::info!("[STRIPE WEBHOOK] User {user_id} access restricted due to unpaid subscription");

                        // Update user subscription status to unpaid in user_subscriptions table
                        update_user_subscription_status(
                            pool,
                            subscription_id,
                            SubscriptionStatus::Unpaid,
                        )
                        .await?;
                        tracing::info!("[STRIPE WEBHOOK] User {user_id} subscription marked as expired due to unpaid status");
                    }
                    "trialing" => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} is in trial for user: {user_id}");
                        // Keep user in trial status
                        crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "trial").await?;
                        tracing::info!("[STRIPE WEBHOOK] User {user_id} kept in trial status");

                        // Update user subscription status to trial in user_subscriptions table
                        update_user_subscription_status(
                            pool,
                            subscription_id,
                            SubscriptionStatus::Trial,
                        )
                        .await?;
                    }
                    _ => {
                        tracing::info!("[STRIPE WEBHOOK] Subscription {subscription_id} has unhandled status: {status} for user: {user_id}");
                        // For unknown statuses, log but don't change user status
                    }
                }
            }
            Err(_) => {
                tracing::error!("[STRIPE WEBHOOK] User not found for customer ID: {customer_id}");
            }
        }
    } else {
        tracing::info!("[STRIPE WEBHOOK] No subscription object found in webhook data");
    }
    Ok(())
}
