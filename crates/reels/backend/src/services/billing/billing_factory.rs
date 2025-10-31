//! Factory for creating billing services with dependency injection.
//!
//! This factory function creates the appropriate billing service implementation
//! based on explicit configuration rather than environment detection. This approach
//! eliminates environment variable race conditions and follows dependency injection
//! principles for better testability and maintainability. Configuration is read
//! once at startup and passed explicitly to avoid global state dependencies.

use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::services::billing::stripe_client::StripeClient;
use crate::services::billing::billing_config::BillingConfig;
use crate::services::billing::billing_service::BillingService;
use crate::services::billing::mock_billing_service::MockBillingService;
use crate::services::billing::billing_service_trait::BillingServiceTrait;

static BILLING_SERVICE: OnceCell<Arc<dyn BillingServiceTrait>> = OnceCell::new();

/// Gets or creates a singleton billing service instance
pub fn get_billing_service() -> Result<&'static Arc<dyn BillingServiceTrait>, String> {
    let config = BillingConfig::from_env();
    BILLING_SERVICE.get_or_try_init(|| create_billing_service(&config))
}

/// Creates the appropriate billing service based on configuration
pub fn create_billing_service(
    config: &crate::services::billing::billing_config::BillingConfig,
) -> std::result::Result<Arc<dyn BillingServiceTrait>, std::string::String> {
    if config.is_test_environment {
        // Test environment - use mock service
        std::result::Result::Ok(Arc::new(MockBillingService::new()))
    } else {
        // Production environment - use real Stripe service
        let stripe_client = StripeClient::new_with_config(config)
            .map_err(|e| std::format!("Failed to create Stripe client: {e}"))?;
        
        let billing_service = BillingService::new_with_client(stripe_client)
            .map_err(|e| std::format!("Failed to create billing service: {e}"))?;
        
        std::result::Result::Ok(Arc::new(billing_service))
    }
}

/// Creates a Stripe client based on configuration
pub fn create_stripe_client(
    config: &crate::services::billing::billing_config::BillingConfig,
) -> std::result::Result<Arc<StripeClient>, std::string::String> {
    let stripe_client = StripeClient::new_with_config(config)
        .map_err(|e| std::format!("Failed to create Stripe client: {e}"))?;
    std::result::Result::Ok(Arc::new(stripe_client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creates_mock_service_in_test_environment() {
        let config = crate::services::billing::billing_config::BillingConfig::for_tests();
        let result = create_billing_service(&config);
        std::assert!(result.is_ok());
    }

    #[test]
    fn test_creates_stripe_service_with_production_config() {
        let config = crate::services::billing::billing_config::BillingConfig::for_production(
            std::string::String::from("sk_test_dummy_key"),
            std::string::String::from("pk_test_dummy_key"),
            std::string::String::from("whsec_test_dummy_secret"),
        );
        let result = create_billing_service(&config);
        std::assert!(result.is_ok());
    }

    #[test]
    fn test_fails_without_secret_key_in_production() {
        let config = crate::services::billing::billing_config::BillingConfig::new(
            std::option::Option::None,
            std::option::Option::Some(std::string::String::from("pk_test_dummy")),
            std::option::Option::Some(std::string::String::from("whsec_test_dummy")),
            false,
        );
        let result = create_billing_service(&config);
        std::assert!(result.is_err());
    }

    #[test]
    fn test_creates_stripe_client_with_config() {
        let config = crate::services::billing::billing_config::BillingConfig::for_tests();
        let result = create_stripe_client(&config);
        std::assert!(result.is_ok());
    }
}
