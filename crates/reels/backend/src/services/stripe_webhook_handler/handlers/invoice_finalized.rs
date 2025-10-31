//! Handler for invoice.finalized Stripe webhook events.
//!
//! This handler processes invoice finalized events by logging the event
//! and extracting relevant information for monitoring purposes.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

/// Handle invoice finalized event
#[instrument(skip(_pool, data))]
pub async fn handle_invoice_finalized_event(
    _pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing invoice.finalized event");

    if let Some(invoice) = data.get("object") {
        let invoice_id = invoice
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let customer_id = invoice
            .get("customer")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = invoice
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!("[STRIPE WEBHOOK] Invoice finalized: {invoice_id} for customer: {customer_id} with status: {status}");
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No invoice object found in webhook data");
    }

    Ok(())
}
