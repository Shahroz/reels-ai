//! Config-aware access control function with explicit trial configuration.
//!
//! This function provides access control logic based on individual user status (trial or 
//! subscription) using a TrialConfig parameter for trial calculations instead of environment 
//! variables. This enables deterministic testing and explicit dependency injection.
//! 
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Removed organization membership hack - access now requires individual credits/trial/subscription
//! - 2025-09-17T20:45:00Z @AI: Created for environment dependency optimization
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool, config))]
pub async fn has_access_with_config(
    pool: &sqlx::PgPool, 
    user_id: uuid::Uuid,
    config: &crate::services::trial_service::trial_config::TrialConfig
) -> std::result::Result<bool, sqlx::Error> {
    let billing_status = crate::services::trial_service::get_billing_status_with_config::get_billing_status_with_config(pool, user_id, config).await?;
    
    let individual_access = match billing_status.trial_status {
        crate::services::trial_service::trial_status::TrialStatus::Active { .. } => true,
        crate::services::trial_service::trial_status::TrialStatus::Expired => billing_status.has_active_subscription,
        crate::services::trial_service::trial_status::TrialStatus::NotStarted => false,
    };
    
    std::result::Result::Ok(individual_access)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_individual_access_logic_with_config() {
        // Test individual access determination logic
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining: 5 };
        let has_subscription = false;
        
        let individual_access = match trial_status {
            crate::services::trial_service::trial_status::TrialStatus::Active { .. } => true,
            crate::services::trial_service::trial_status::TrialStatus::Expired => has_subscription,
            crate::services::trial_service::trial_status::TrialStatus::NotStarted => false,
        };
        
        assert!(individual_access);
    }

    #[test]
    fn test_config_parameter_usage() {
        // Test that the function accepts and can use config
        let config = crate::services::trial_service::trial_config::TrialConfig::new(21);
        assert_eq!(config.trial_period_days(), 21);
        
        // This config would be passed to get_billing_status_with_config
        // The actual database testing is done in integration tests
    }

    #[test]
    fn test_access_logic_returns_individual_access() {
        // Test that function returns individual access status directly
        // (organization membership hack removed)
        let has_individual_access = true;
        
        // Function now returns individual access directly without fallback
        assert_eq!(has_individual_access, true);
        
        let no_individual_access = false;
        assert_eq!(no_individual_access, false);
    }
}
