#![allow(clippy::disallowed_methods)]
//! Check valid product type for the user's active subscription.
//!
//! Returns true if product type is valid, false otherwise and error if product type is not valid.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::user_subscription_schemas::SubscriptionStatus;

/// Check valid product type for the user's active subscription
/// Returns true if product type is valid, false otherwise and error if product type is not valid
#[instrument(skip(pool))]
pub async fn check_valid_product_type(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(bool, Option<Error>), Error> {

    // Getting the product type from environment
    let stripe_metadata_product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default();
    if stripe_metadata_product_type.is_empty() {
        let err_msg = format!("[USER CREDIT ALLOCATION] STRIPE_METADATA_PRODUCT_TYPE environment variable not set");
        return Ok((false, Some(Error::Protocol(err_msg.into()))));
    }

    // Getting the current product id from user active subscription
    let user_subscription = crate::queries::user_subscription::get_current_active_subscription::get_current_active_subscription(pool, user_id).await?;
    let user_subscription = user_subscription.ok_or_else(|| Error::RowNotFound)?;
    // Checking if the user has an active subscription
    if user_subscription.status != SubscriptionStatus::Trialing && user_subscription.status != SubscriptionStatus::Trial {
        return Ok((false, Some(Error::Protocol("User does not have an active subscription".into()))));
    }

    // Getting the product id from the user subscription
    let product_id = user_subscription.stripe_product_id;

    // Getting the billing service
    let billing_service = crate::services::billing::billing_factory::get_billing_service().map_err(|e| Error::Protocol(e.into()))?;

    // Getting product details from billing service
    let product = billing_service.get_product(&product_id, false).await.map_err(|e| Error::Protocol(e.to_string().into()))?;
    let product_type = product.metadata.get("product_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    // Checking if the product type is matched with the environment variable
    if product_type != stripe_metadata_product_type {
        return Ok((false, Some(Error::Protocol(format!("Product type does not match the environment variable, expected: {}, got: {}", stripe_metadata_product_type, product_type).into()))));
    }

    // Returning true if product type is valid
    Ok((true, None))
}
