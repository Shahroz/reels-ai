//! Handler for customer.subscription.deleted Stripe webhook events.
//!
//! This handler processes subscription deleted events by checking if the user
//! has any remaining active subscriptions and revoking access if none exist.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use crate::queries::user_subscription::{get_user_subscription_by_stripe_id, update_user_subscription_status};

/// Handle subscription deleted event
#[instrument(skip(billing_service, pool, data))]
pub async fn handle_subscription_deleted_event(
    billing_service: &dyn BillingServiceTrait,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing customer.subscription.deleted event");

    if let Some(subscription) = data.get("object") {
        let customer_id = subscription
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let subscription_id = subscription
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing subscription ID"))?;

        tracing::info!("[STRIPE WEBHOOK] Subscription deleted: {subscription_id} for customer: {customer_id}");

        // Check if subscription is in user_subscriptions table
        let user_subscription = get_user_subscription_by_stripe_id(pool, subscription_id).await?;
        if user_subscription.is_none() {
            tracing::info!("[STRIPE WEBHOOK] Subscription ID: {subscription_id} not found in our records, skipping subscription deletion");
            return Ok(());
        }

        // Extract the first price from subscription items to get product information
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
            tracing::info!("[STRIPE WEBHOOK] Product plan is free, skipping subscription deletion");
            return Ok(());
        }

        // Only proceed if product_type matches the environment variable
        if product_type != stripe_metadata_product_type {
            tracing::info!("[STRIPE WEBHOOK] Product type does not match, skipping subscription deletion");
            return Ok(());
        }

        tracing::info!("[STRIPE WEBHOOK] Product type matches, proceeding with subscription deletion");

        // Find user by Stripe customer ID
        let user_id = match crate::queries::webhooks::users::get_user_id_by_stripe_customer_id(pool, customer_id).await? {
            Some(id) => id,
            None => {
                tracing::error!("[STRIPE WEBHOOK] User not found for customer ID: {customer_id}");
                return Ok(());
            }
        };

        // Expiring the subscription in user_subscriptions table
        update_user_subscription_status(
            pool,
            subscription_id,
            SubscriptionStatus::Expired, // Expiring the subscription
        )
        .await?;

        // Checking if user has any active subscription count greater than 0
        let active_count = crate::queries::webhooks::subscriptions::get_active_subscription_count_for_user(pool, user_id).await?;

        if active_count == 0 {
            // Revoke access due to subscription deletion
            tracing::info!("[STRIPE WEBHOOK] Revoking access for user: {user_id} due to subscription deletion");

            // Update user subscription status to expired and clear stripe_customer_id
            crate::queries::webhooks::users::expire_user_subscription_and_clear_stripe_id(pool, user_id).await?;

            tracing::info!("[STRIPE WEBHOOK] User {user_id} access revoked and subscription marked as expired");

            // Track subscription canceled analytics (non-blocking) - scheduled cancellation (period end)
            crate::routes::stripe::analytics_helper::track_subscription_canceled(
                pool, user_id,
                "expired", // Previous status was expired before deletion (period end reached)
                true,      // This is a scheduled cancellation that reached period end
            )
            .await;
        }
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No subscription object found in webhook data");
    }

    Ok(())
}
