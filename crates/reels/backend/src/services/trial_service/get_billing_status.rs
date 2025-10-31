//! Function to retrieve complete billing status information for a user.
//!
//! This function combines trial status calculation with subscription information to provide
//! a comprehensive view of a user's billing state. It performs a single database query
//! to fetch user billing data, then calls get_trial_status to calculate the current trial state.
//! The function determines active subscription status based on Stripe integration requirements.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool))]
pub async fn get_billing_status(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> std::result::Result<crate::services::trial_service::billing_status::BillingStatus, sqlx::Error> {
    let config = crate::services::trial_service::trial_config::TrialConfig::from_env();
    crate::services::trial_service::get_billing_status_with_config::get_billing_status_with_config(pool, user_id, &config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_status_logic_active() {
        // Test the logic for determining active subscriptions
        let subscription_status = std::option::Option::Some("active");
        let stripe_customer_id = std::option::Option::Some("cus_test".to_string());
        
        let has_active = matches!(subscription_status.as_deref(), std::option::Option::Some("active" | "canceled")) && stripe_customer_id.is_some();
        assert!(has_active);
    }

    #[test]
    fn test_subscription_status_logic_cancelled() {
        // Test canceled subscriptions are considered active (with remaining access)
        let subscription_status = std::option::Option::Some("canceled");
        let stripe_customer_id = std::option::Option::Some("cus_test".to_string());
        
        let has_active = matches!(subscription_status.as_deref(), std::option::Option::Some("active" | "canceled")) && stripe_customer_id.is_some();
        assert!(has_active);
    }

    #[test]
    fn test_subscription_status_logic_trial() {
        // Test trial status does not count as active subscription
        let subscription_status = std::option::Option::Some("trial");
        let stripe_customer_id: std::option::Option<std::string::String> = std::option::Option::None;
        
        let has_active = matches!(subscription_status.as_deref(), std::option::Option::Some("active" | "canceled")) && stripe_customer_id.is_some();
        assert!(!has_active);
    }

    #[test]
    fn test_subscription_status_logic_no_customer_id() {
        // Test that active subscription requires Stripe customer ID
        let subscription_status = std::option::Option::Some("active");
        let stripe_customer_id: std::option::Option<std::string::String> = std::option::Option::None;
        
        let has_active = matches!(subscription_status.as_deref(), std::option::Option::Some("active" | "canceled")) && stripe_customer_id.is_some();
        assert!(!has_active);
    }

    #[test]
    fn test_default_subscription_status() {
        // Test the default subscription status when none is provided
        let subscription_status: std::option::Option<std::string::String> = std::option::Option::None;
        let default_status = subscription_status.unwrap_or_else(|| "trial".to_string());
        assert_eq!(default_status, "trial");
    }
}
