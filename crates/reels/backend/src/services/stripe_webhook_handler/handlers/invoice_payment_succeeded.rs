//! Handler for invoice.payment_succeeded Stripe webhook events.
//!
//! This handler processes invoice payment succeeded events by creating
//! payment completions and updating user credit allocations.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

use crate::db::payment_completions::create_payment_completion;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::services::billing::billing_service_trait::BillingServiceTrait;

/// Handle invoice payment succeeded event
#[instrument(skip(_billing_service, pool, data))]
pub async fn handle_invoice_payment_succeeded_event(
    _billing_service: &dyn BillingServiceTrait,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing invoice.payment_succeeded event");

    if let Some(invoice) = data.get("object") {
        let invoice_id = invoice
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing invoice ID"))?;

        let customer_id = invoice
            .get("customer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing customer ID"))?;

        let _subscription_id = invoice.get("subscription").and_then(|v| v.as_str()); // Marked as unused with _
        let amount_paid = invoice
            .get("amount_paid")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Extract product information from invoice lines
        let product_id = invoice
            .get("lines")
            .and_then(|lines| lines.get("data"))
            .and_then(|data| data.as_array())
            .and_then(|arr| arr.first())
            .and_then(|line| line.get("price"))
            .and_then(|price| price.get("product"))
            .and_then(|product| product.as_str());

        let product_plan = invoice
            .get("lines")
            .and_then(|lines| lines.get("data"))
            .and_then(|data| data.as_array())
            .and_then(|arr| arr.first())
            .and_then(|line| line.get("price"))
            .and_then(|price| price.get("product"))
            .and_then(|product| product.get("metadata"))
            .and_then(|metadata| metadata.get("product_plan"))
            .and_then(|plan| plan.as_str())
            .unwrap_or("unknown");

        let product_type = invoice
            .get("lines")
            .and_then(|lines| lines.get("data"))
            .and_then(|data| data.as_array())
            .and_then(|arr| arr.first())
            .and_then(|line| line.get("price"))
            .and_then(|price| price.get("product"))
            .and_then(|product| product.get("metadata"))
            .and_then(|metadata| metadata.get("product_type"))
            .and_then(|type_val| type_val.as_str())
            .unwrap_or("unknown");

        tracing::info!("[STRIPE WEBHOOK] Invoice payment succeeded: {invoice_id} for customer: {customer_id} (amount: {amount_paid}, product: {product_id:?})");

        // Find user by Stripe customer ID
        let user_id = match crate::queries::webhooks::users::get_user_id_by_stripe_customer_id(pool, customer_id).await? {
            Some(id) => id,
            None => {
                tracing::warn!("[STRIPE WEBHOOK] No user found for customer: {customer_id}");
                return Ok(());
            }
        };

        // Handle different product types
        if product_plan != "unknown" && product_type != "unknown" {
            if product_plan == StripePlanType::Free.as_str() {
                // Check if payment completion already exists
                let existing_payment = crate::queries::webhooks::payments::payment_completion_exists_by_session_id(pool, invoice_id).await?;

                if !existing_payment {
                    tracing::info!("[STRIPE WEBHOOK] Product plan is free, adding payment completion");
                    create_payment_completion(pool, user_id, invoice_id, "free", amount_paid as i32, "usd", None).await?;
                }
            }
        }
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No invoice object found in webhook data");
    }

    Ok(())
}
