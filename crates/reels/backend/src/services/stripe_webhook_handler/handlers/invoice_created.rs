//! Handler for invoice.created Stripe webhook events.
//!
//! This handler processes invoice created events by logging the event
//! and extracting relevant information for monitoring purposes.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

/// Handle invoice created event
#[instrument(skip(_pool, data))]
pub async fn handle_invoice_created_event(
    _pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing invoice.created event");

    if let Some(invoice) = data.get("object") {
        let customer_id = invoice
            .get("customer")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let invoice_id = invoice
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let billing_reason = invoice
            .get("billing_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Extract additional invoice information for better monitoring
        let status = invoice
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let amount_due = invoice
            .get("amount_due")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let currency = invoice
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let subscription_id = invoice
            .get("subscription")
            .and_then(|v| v.as_str())
            .unwrap_or("none");

        let period_start = invoice
            .get("period_start")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let period_end = invoice
            .get("period_end")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        tracing::info!(
            "[STRIPE WEBHOOK] Invoice created: {invoice_id} for customer: {customer_id} (reason: {billing_reason}, status: {status}, amount: {amount_due} {currency}, subscription: {subscription_id}, period: {period_start}-{period_end})"
        );

        // Log invoice lines for detailed monitoring
        if let Some(lines) = invoice.get("lines") {
            if let Some(data) = lines.get("data") {
                if let Some(items) = data.as_array() {
                    tracing::info!("[STRIPE WEBHOOK] Invoice {invoice_id} has {} line items", items.len());
                    
                    for (index, item) in items.iter().enumerate() {
                        if let Some(description) = item.get("description").and_then(|v| v.as_str()) {
                            let amount = item.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
                            tracing::info!("[STRIPE WEBHOOK] Line item {}: {} (amount: {})", index + 1, description, amount);
                        }
                    }
                }
            }
        }

        // Optional: Add custom invoice items or billing logic here
        // This event precedes actual payment
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No invoice object found in webhook data");
    }

    Ok(())
}
