//! Organization webhook query functions.
//!
//! This module provides database queries specifically designed for webhook processing
//! that need to look up organizations by their Stripe customer ID.
//! These are kept separate from the main organization queries to maintain clear boundaries
//! between webhook-specific and general-purpose query logic.

use sqlx::PgPool;
use uuid::Uuid;

/// Get organization ID by stripe_customer_id
///
/// Used by webhook handlers to find which organization to update
/// when receiving Stripe events (subscription created, invoice paid, etc.)
#[tracing::instrument(skip(pool))]
pub async fn get_organization_id_by_stripe_customer_id(
    pool: &PgPool,
    stripe_customer_id: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT id FROM organizations WHERE stripe_customer_id = $1",
        stripe_customer_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.id))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // This test verifies the module compiles
        assert!(true);
    }
}

