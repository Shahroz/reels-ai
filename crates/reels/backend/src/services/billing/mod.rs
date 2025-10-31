//! Billing service module organization and re-exports.
//!
//! This module implements billing and Stripe services with dependency injection
//! for better testability and maintainability.
//! 
//! ## Architecture
//! - **BillingServiceTrait**: Core trait defining the billing interface
//! - **BillingService**: Production implementation using Stripe APIs
//! - **MockBillingService**: Mock implementation for testing
//! - **BillingConfig**: Configuration struct for dependency injection
//! - **billing_factory**: Factory functions using explicit configuration
//! - **StripeClient**: Low-level Stripe API client
//! - **Environment-aware**: Uses APP_ENV for proper service selection
//! - **Dependency injection**: Explicit configuration prevents race conditions
//! - **Testable**: Mock service for testing without external API calls
//!
//! ## Usage
//! ```ignore
//! // Recommended: Use explicit configuration
//! let config = crate::services::billing::billing_config::BillingConfig::from_env();
//! let billing_service = crate::services::billing::billing_factory::create_billing_service(&config)?;
//! let stripe_client = crate::services::billing::billing_factory::create_stripe_client(&config)?;
//!
//! // Environment mapping:
//! // - APP_ENV=test: Returns MockBillingService with dummy keys
//! // - APP_ENV=dev/prod: Returns BillingService with real Stripe integration
//! ```

pub mod billing_config;
pub mod billing_service;
pub mod billing_service_trait;
pub mod mock_billing_service;
pub mod billing_factory;
pub mod stripe_client;
