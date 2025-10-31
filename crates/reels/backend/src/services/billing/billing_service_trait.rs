//! Trait for billing services to enable dependency injection and testing.
//!
//! This trait provides a common interface for billing functionality,
//! enabling dependency injection and testability in the application.
//! Implementations can vary from production services using Stripe APIs
//! to mock services returning dummy data for testing scenarios.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use async_trait::async_trait;

use crate::services::billing::stripe_client::{StripeCancellationDetails, StripeCustomer, StripePlanWithProduct, StripeProductWithPrices, StripePrice, StripeSubscriptionList};
use crate::services::billing::billing_service::{
    CheckoutSessionResponse, CustomerPortalResponse
};

/// Trait for billing services
#[async_trait]
pub trait BillingServiceTrait: Send + Sync {
    /// Fetch Stripe products tagged with optionally filtered by meta_product_type
    async fn get_products(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripeProductWithPrices>>;

    /// Get a specific product by ID
    async fn get_product(&self, product_id: &str, with_prices: bool) -> Result<StripeProductWithPrices>;

    /// Get a specific price by ID
    async fn get_price(&self, price_id: &str) -> Result<StripePrice>;

    /// Fetch Stripe plans with product details
    async fn get_plans(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripePlanWithProduct>>;

    /// Create a free subscription for a user
    async fn create_free_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<()>;

    /// Create a Stripe checkout session
    async fn create_checkout_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
    ) -> Result<CheckoutSessionResponse>;

    /// Create a Stripe checkout session with customer type and optional organization context
    async fn create_checkout_session_with_context(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
        customer_type: &str,
        organization_id: Option<Uuid>,
    ) -> Result<CheckoutSessionResponse>;

    /// Create a customer portal session
    async fn create_customer_portal_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        return_url: &str,
    ) -> Result<CustomerPortalResponse>;

    /// Update checkout session status
    async fn update_checkout_session_status(
        &self,
        pool: &PgPool,
        stripe_checkout_id: &str,
        status: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()>;

    /// Activate user subscription
    async fn activate_user_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        stripe_customer_id: &str,
    ) -> Result<()>;

    /// Cancel a Stripe subscription and update local database
    async fn cancel_subscription(
        &self,
        pool: &PgPool,
        subscription_id: &str,
        cancellation_details: Option<StripeCancellationDetails>,
        invoice_now: Option<bool>,
        prorate: Option<bool>,
    ) -> Result<()>;

    /// Get subscriptions by customer ID, filtered by products matching STRIPE_METADATA_PRODUCT_TYPE
    async fn get_subscriptions_by_customer(&self, customer_id: &str) -> Result<StripeSubscriptionList>;

    /// Get a customer by ID
    async fn get_customer(&self, customer_id: &str) -> Result<StripeCustomer>;

    /// Update a customer's email address
    /// 
    /// Note: The customer ID remains the same - only the email is updated.
    /// This is useful for organization ownership transfers.
    async fn update_customer_email(&self, customer_id: &str, new_email: &str) -> Result<StripeCustomer>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::billing::mock_billing_service::MockBillingService;

    #[tokio::test]
    async fn test_mock_billing_service_implements_trait() {
        let service = MockBillingService::new();
        
        // This should compile if MockBillingService implements BillingServiceTrait
        let _: &dyn BillingServiceTrait = &service;
    }
}
