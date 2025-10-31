//! Service for handling Stripe webhook events.
//!
//! This service handles Stripe webhook events by processing different event types
//! and delegating to appropriate handlers. It follows the service pattern for
//! external integrations and business logic.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;
use std::sync::Arc;

use crate::services::billing::billing_config::BillingConfig;
use crate::services::billing::billing_factory::create_billing_service;
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use crate::services::stripe_webhook_handler::handlers::*;

/// Stripe webhook events handler service
pub struct StripeWebhookEventsHandlerService {
    billing_config: BillingConfig,
    billing_service: Arc<dyn BillingServiceTrait>,
}

impl StripeWebhookEventsHandlerService {
    /// Create a new Stripe webhook events handler service
    pub fn new() -> Result<Self> {
        let billing_config = BillingConfig::from_env();
        let billing_service = create_billing_service(&billing_config)
            .map_err(|e| anyhow::anyhow!("Failed to create billing service: {}", e))?;
        
        Ok(Self { 
            billing_config,
            billing_service,
        })
    }

    /// Process checkout session completed event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_checkout_session_completed(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_checkout_session_completed_event(self.billing_service.as_ref(), pool, data).await
    }

    /// Process subscription created event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_subscription_created(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_subscription_created_event(self.billing_service.as_ref(), pool, data).await
    }

    /// Process subscription updated event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_subscription_updated(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_subscription_updated_event(self.billing_service.as_ref(), pool, data).await
    }

        /// Process subscription deleted event
        #[instrument(skip(self, pool, data))]
        pub async fn handle_subscription_deleted(
            &self,
            pool: &PgPool,
            data: &serde_json::Value,
        ) -> Result<()> {
            handle_subscription_deleted_event(self.billing_service.as_ref(), pool, data).await
        }

    /// Process invoice payment succeeded event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_invoice_payment_succeeded(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_invoice_payment_succeeded_event(self.billing_service.as_ref(), pool, data).await
    }

    /// Process invoice payment failed event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_invoice_payment_failed(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_invoice_payment_failed_event(pool, data).await
    }

    /// Process product updated event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_product_updated(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_product_updated_event(pool, data).await
    }

    /// Process invoice created event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_invoice_created(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_invoice_created_event(pool, data).await
    }

    /// Process invoice finalized event
    #[instrument(skip(self, pool, data))]
    pub async fn handle_invoice_finalized(
        &self,
        pool: &PgPool,
        data: &serde_json::Value,
    ) -> Result<()> {
        handle_invoice_finalized_event(pool, data).await
    }

        /// Process invoice paid event
        #[instrument(skip(self, pool, data))]
        pub async fn handle_invoice_paid(
            &self,
            pool: &PgPool,
            data: &serde_json::Value,
        ) -> Result<()> {
            handle_invoice_paid_event(self.billing_service.as_ref(), pool, data).await
        }
}