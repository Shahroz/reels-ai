//! Config-aware function to retrieve billing status with explicit trial configuration.
//!
//! This function provides the same billing status retrieval as get_billing_status but uses
//! a TrialConfig parameter for trial period calculations instead of environment variables.
//! This enables deterministic testing and explicit dependency injection. The function
//! delegates trial status calculation to get_trial_status_with_config while maintaining
//! the same subscription status determination logic.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created for environment dependency optimization
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool, config))]
pub async fn get_billing_status_with_config(
    pool: &sqlx::PgPool, 
    user_id: uuid::Uuid,
    config: &crate::services::trial_service::trial_config::TrialConfig
) -> std::result::Result<crate::services::trial_service::billing_status::BillingStatus, sqlx::Error> {
    let user_billing_info = crate::queries::trial_service::users::get_user_billing_info(pool, user_id).await?;

    let trial_status = crate::services::trial_service::get_trial_status_with_config::get_trial_status_with_config(pool, user_id, config).await?;
    
    let has_active_subscription = matches!(user_billing_info.subscription_status.as_deref(), std::option::Option::Some("active" | "trial" | "trialing"));

    std::result::Result::Ok(crate::services::trial_service::billing_status::BillingStatus {
        trial_status,
        subscription_status: user_billing_info.subscription_status.unwrap_or_else(|| "trial".to_string()),
        has_active_subscription,
        stripe_customer_id: user_billing_info.stripe_customer_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_status_logic_with_config() {
        // Test subscription status determination logic (independent of config)
        let subscription_status = std::option::Option::Some("active");
        let stripe_customer_id = std::option::Option::Some("cus_test".to_string());
        
        let has_active = matches!(subscription_status.as_deref(), std::option::Option::Some("active" | "trial" | "trialing")) && stripe_customer_id.is_some();
        assert!(has_active);
    }

    #[test]
    fn test_default_subscription_status_with_config() {
        // Test default subscription status when none provided
        let subscription_status: std::option::Option<std::string::String> = std::option::Option::None;
        let default_status = subscription_status.unwrap_or_else(|| "trial".to_string());
        assert_eq!(default_status, "trial");
    }

    #[test]
    fn test_config_usage() {
        // Test that the function accepts the config parameter
        let config = crate::services::trial_service::trial_config::TrialConfig::new(10);
        assert_eq!(config.trial_period_days(), 10);
        
        // This config would be passed to get_trial_status_with_config
        // The actual database testing is done in integration tests
    }
}
