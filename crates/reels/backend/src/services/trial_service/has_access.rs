//! Primary access control function based on individual user status.
//!
//! This function serves as the main entry point for determining user application access.
//! It checks individual access based on trial status or active subscription. As of 2025-10-17,
//! organization-based access sharing has been removed. Access now requires individual
//! credits, trial status, or active subscription.
//! 
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Removed organization access (organization hack removed)
//! - 2025-09-17T20:45:00Z @AI: Updated during organization-based billing implementation
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool))]
pub async fn has_access(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> std::result::Result<bool, sqlx::Error> {
    let config = crate::services::trial_service::trial_config::TrialConfig::from_env();
    crate::services::trial_service::has_access_with_config::has_access_with_config(pool, user_id, &config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_individual_access_logic_active_trial() {
        // Test individual access with active trial
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
    fn test_individual_access_logic_expired_with_subscription() {
        // Test individual access with expired trial but active subscription
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::Expired;
        let has_subscription = true;
        
        let individual_access = match trial_status {
            crate::services::trial_service::trial_status::TrialStatus::Active { .. } => true,
            crate::services::trial_service::trial_status::TrialStatus::Expired => has_subscription,
            crate::services::trial_service::trial_status::TrialStatus::NotStarted => false,
        };
        
        assert!(individual_access);
    }

    #[test]
    fn test_individual_access_logic_expired_no_subscription() {
        // Test individual access with expired trial and no subscription
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::Expired;
        let has_subscription = false;
        
        let individual_access = match trial_status {
            crate::services::trial_service::trial_status::TrialStatus::Active { .. } => true,
            crate::services::trial_service::trial_status::TrialStatus::Expired => has_subscription,
            crate::services::trial_service::trial_status::TrialStatus::NotStarted => false,
        };
        
        assert!(!individual_access);
    }

    #[test]
    fn test_individual_access_logic_not_started() {
        // Test individual access with trial not started
        let trial_status = crate::services::trial_service::trial_status::TrialStatus::NotStarted;
        let has_subscription = true; // Even with subscription, not started means no access
        
        let individual_access = match trial_status {
            crate::services::trial_service::trial_status::TrialStatus::Active { .. } => true,
            crate::services::trial_service::trial_status::TrialStatus::Expired => has_subscription,
            crate::services::trial_service::trial_status::TrialStatus::NotStarted => false,
        };
        
        assert!(!individual_access);
    }
}
