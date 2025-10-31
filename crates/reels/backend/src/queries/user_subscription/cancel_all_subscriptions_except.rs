#![allow(clippy::disallowed_methods)]
//! Cancel all subscriptions for a customer except the one with the given subscription ID.
//!
//! This function fetches all subscriptions for a customer using the billing service,
//! then cancels all subscriptions except the one specified by `keep_subscription_id`.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use crate::services::billing::stripe_client::{StripeCancellationDetails, StripeCancellationFeedback};

/// Cancel all subscriptions for a customer except the one with the given subscription ID
/// 
/// This function fetches all subscriptions for a customer using the billing service,
/// then cancels all subscriptions except the one specified by `keep_subscription_id`.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `billing_service` - Billing service instance for Stripe API calls
/// * `customer_id` - Stripe customer ID
/// * `keep_subscription_id` - Subscription ID to keep active (all others will be canceled)
/// 
/// # Returns
/// * `Result<Vec<String>, Error>` - List of canceled subscription IDs
#[instrument(skip(pool, billing_service))]
pub async fn cancel_all_subscriptions_except(
    pool: &PgPool,
    user_id: Uuid,
    customer_id: &str,
    keep_subscription_id: &str,
    billing_service: &dyn BillingServiceTrait,
) -> Result<Vec<String>, Error> {
    log::info!(
        "Canceling all subscriptions for customer {} except subscription {}",
        customer_id,
        keep_subscription_id
    );

    // Updating all active user subscriptions status to canceled, except the one to keep
    sqlx::query!(
        r#"
            UPDATE user_subscriptions 
            SET status = 'canceled' 
            WHERE user_id = $1 
                AND stripe_subscription_id != $2 
                AND status NOT IN ('canceled', 'expired')
        "#,
        user_id,
        keep_subscription_id,
    )
    .execute(pool)
    .await?;

    // Fetch all subscriptions for the customer
    let subscriptions_response = billing_service
        .get_subscriptions_by_customer(customer_id)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch subscriptions for customer {}: {}", customer_id, e);
            Error::Protocol(format!("Failed to fetch subscriptions: {}", e))
        })?;

    let mut canceled_subscription_ids = Vec::new();

    // Cancel all subscriptions except the one to keep
    for subscription in subscriptions_response.data {
        if subscription.id != keep_subscription_id {
            log::info!(
                "Canceling subscription {} for customer {}",
                subscription.id,
                customer_id
            );

            // Cancel the subscription using the billing service
            let cancellation_details = StripeCancellationDetails {
                comment: Some("Canceled to keep only one active subscription".to_string()),
                feedback: Some(StripeCancellationFeedback::Other),
            };

            match billing_service
                .cancel_subscription(
                    pool,
                    &subscription.id,
                    Some(cancellation_details),
                    Some(false), // Don't invoice immediately
                    Some(false), // Don't prorate
                )
                .await
            {
                Ok(_) => {
                    log::info!("Successfully canceled subscription {}", subscription.id);
                    canceled_subscription_ids.push(subscription.id);
                }
                Err(e) => {
                    log::error!(
                        "Failed to cancel subscription {}: {}",
                        subscription.id,
                        e
                    );
                    // Continue with other subscriptions even if one fails
                }
            }
        } else {
            log::info!(
                "Keeping subscription {} active for customer {}",
                subscription.id,
                customer_id
            );
        }
    }

    log::info!(
        "Canceled {} subscriptions for customer {} (kept subscription {})",
        canceled_subscription_ids.len(),
        customer_id,
        keep_subscription_id
    );

    Ok(canceled_subscription_ids)
}
