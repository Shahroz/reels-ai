//! Configuration for billing services.
//!
//! This struct holds configuration data for billing service creation,
//! including API keys and environment detection. Uses APP_ENV to determine
//! the runtime environment (test/dev/prod) for proper service selection.
//! By reading environment variables once at startup and passing configuration
//! explicitly, we avoid environment variable race conditions in tests and follow
//! dependency injection principles for better testability and maintainability.
//!
//! Revision History:
//! - 2025-01-XX: Created billing configuration to replace STORAGE_EMULATOR_HOST detection
//! - Prior revision history not available

/// Configuration for billing service creation
#[derive(Clone)]
pub struct BillingConfig {
    /// Stripe secret key for production billing service
    stripe_secret_key: std::option::Option<std::string::String>,
    /// Stripe publishable key for frontend integration
    stripe_publishable_key: std::option::Option<std::string::String>,
    /// Stripe webhook secret for webhook validation
    stripe_webhook_secret: std::option::Option<std::string::String>,
    /// Whether we're running in a test environment (APP_ENV=test)
    pub is_test_environment: bool,
}

impl std::fmt::Debug for BillingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BillingConfig")
            .field("has_secret_key", &self.stripe_secret_key.is_some())
            .field("has_publishable_key", &self.stripe_publishable_key.is_some())
            .field("has_webhook_secret", &self.stripe_webhook_secret.is_some())
            .field("is_test_environment", &self.is_test_environment)
            .finish()
    }
}

impl BillingConfig {
    /// Create a new billing configuration
    pub fn new(
        stripe_secret_key: std::option::Option<std::string::String>,
        stripe_publishable_key: std::option::Option<std::string::String>,
        stripe_webhook_secret: std::option::Option<std::string::String>,
        is_test_environment: bool,
    ) -> Self {
        Self {
            stripe_secret_key,
            stripe_publishable_key,
            stripe_webhook_secret,
            is_test_environment,
        }
    }
    
    /// Create configuration from environment variables (called once at startup)
    /// 
    /// # Panics
    /// Panics if APP_ENV environment variable is not set. This is intentional to prevent
    /// accidental deployment without explicit environment configuration.
    pub fn from_env() -> Self {
        let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY").ok();
        let stripe_publishable_key = std::env::var("STRIPE_PUBLISHABLE_KEY").ok();
        let stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();
        let app_env = std::env::var("APP_ENV")
            .expect("APP_ENV environment variable must be set (test/dev/prod)");
        let is_test_environment = app_env == "test";
        
        Self::new(stripe_secret_key, stripe_publishable_key, stripe_webhook_secret, is_test_environment)
    }
    
    /// Create test configuration with dummy keys
    pub fn for_tests() -> Self {
        Self::new(
            std::option::Option::Some(std::string::String::from("sk_test_dummy_key_for_testing")),
            std::option::Option::Some(std::string::String::from("pk_test_dummy_key_for_testing")),
            std::option::Option::Some(std::string::String::from("whsec_test_dummy_secret_for_testing")),
            true,
        )
    }
    
    /// Create production configuration with provided keys
    pub fn for_production(
        secret_key: std::string::String,
        publishable_key: std::string::String,
        webhook_secret: std::string::String,
    ) -> Self {
        Self::new(
            std::option::Option::Some(secret_key),
            std::option::Option::Some(publishable_key),
            std::option::Option::Some(webhook_secret),
            false,
        )
    }
    
    /// Get the secret key, using dummy key for tests
    pub fn get_secret_key(&self) -> std::result::Result<std::string::String, std::string::String> {
        if self.is_test_environment {
            std::result::Result::Ok(std::string::String::from("sk_test_dummy_key_for_testing"))
        } else {
            self.stripe_secret_key.clone()
                .ok_or_else(|| std::string::String::from("STRIPE_SECRET_KEY not provided in production configuration"))
        }
    }
    
    /// Get the publishable key, using dummy key for tests
    pub fn get_publishable_key(&self) -> std::result::Result<std::string::String, std::string::String> {
        if self.is_test_environment {
            std::result::Result::Ok(std::string::String::from("pk_test_dummy_key_for_testing"))
        } else {
            self.stripe_publishable_key.clone()
                .ok_or_else(|| std::string::String::from("STRIPE_PUBLISHABLE_KEY not provided in production configuration"))
        }
    }
    
    /// Get the webhook secret, using dummy secret for tests
    pub fn get_webhook_secret(&self) -> std::result::Result<std::string::String, std::string::String> {
        if self.is_test_environment {
            std::result::Result::Ok(std::string::String::from("whsec_test_dummy_secret_for_testing"))
        } else {
            self.stripe_webhook_secret.clone()
                .ok_or_else(|| std::string::String::from("STRIPE_WEBHOOK_SECRET not provided in production configuration"))
        }
    }
}
