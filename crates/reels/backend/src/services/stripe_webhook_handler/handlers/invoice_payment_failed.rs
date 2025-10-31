//! Handler for invoice.payment_failed Stripe webhook events.
//!
//! This handler processes invoice payment failed events by updating
//! user subscription status based on failure attempts.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

/// Handle invoice payment failed event
#[instrument(skip(pool, data))]
pub async fn handle_invoice_payment_failed_event(
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing invoice.payment_failed event");

    if let Some(invoice) = data.get("object") {
        let invoice_id = invoice
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing invoice ID"))?;

        let customer_id = invoice
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let status = invoice
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let attempt_count = invoice
            .get("attempt_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        tracing::info!("[STRIPE WEBHOOK] Invoice payment failed: {invoice_id} for customer: {customer_id} (status: {status}, attempts: {attempt_count})");

        // Find user by Stripe customer ID
        let user_id = match crate::queries::webhooks::users::get_user_id_by_stripe_customer_id(pool, customer_id).await? {
            Some(id) => id,
            None => {
                tracing::warn!("[STRIPE WEBHOOK] No user found for customer: {customer_id}");
                return Ok(());
            }
        };

        // Update user status based on failure
        if attempt_count >= 3 {
            tracing::info!("[STRIPE WEBHOOK] Payment failed after {attempt_count} attempts, expiring user: {user_id}");
            crate::queries::webhooks::users::update_subscription_status_in_user(pool, user_id, "expired").await?;
        }
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No invoice object found in webhook data");
    }

    Ok(())
}
