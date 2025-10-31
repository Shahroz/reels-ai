//! Handler for product.updated Stripe webhook events.
//!
//! This handler processes product updated events by updating subscription
//! plan types for all users with the affected product.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;

use crate::schemas::user_credit_allocation_schemas::StripePlanType;

/// Handle product updated event
#[instrument(skip(pool, data))]
pub async fn handle_product_updated_event(
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<()> {
    tracing::info!("[STRIPE WEBHOOK] Processing product.updated event");

    if let Some(product) = data.get("object") {
        let product_id = product
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing product ID"))?;

        let product_plan = product
            .get("metadata")
            .and_then(|metadata| metadata.get("product_plan"))
            .and_then(|plan| plan.as_str())
            .unwrap_or("unknown");

        let product_type = product
            .get("metadata")
            .and_then(|metadata| metadata.get("product_type"))
            .and_then(|type_val| type_val.as_str())
            .unwrap_or("unknown");

        tracing::info!("[STRIPE WEBHOOK] Product {product_id} metadata: product_plan={product_plan}, product_type={product_type}");

        // Check if there are any subscriptions using this product
        let subscription_count = crate::queries::webhooks::subscriptions::get_subscription_count_by_stripe_product_id(pool, product_id).await?;

        if subscription_count > 0 {
            let credit_plan_type = StripePlanType::from_str(product_plan);
            if credit_plan_type != StripePlanType::Unknown {
                // Bulk update stripe_plan_type for all users with this product
                crate::queries::webhooks::subscriptions::update_stripe_plan_type_by_product_id(pool, product_id, product_plan).await?;
            }
        }
    } else {
        tracing::warn!("[STRIPE WEBHOOK] No product object found in webhook data");
    }

    Ok(())
}
