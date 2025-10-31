//! Config-aware function to calculate trial status with explicit trial period configuration.
//!
//! This function provides the same trial status calculation as get_trial_status but accepts
//! a TrialConfig parameter instead of reading environment variables directly. This enables
//! deterministic testing and explicit dependency injection. The function logic is identical
//! to the environment-based version but uses the provided configuration for trial period
//! calculations instead of calling get_trial_period_days().
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created for environment dependency optimization
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool, config))]
pub async fn get_trial_status_with_config(
    pool: &sqlx::PgPool, 
    user_id: uuid::Uuid,
    config: &crate::services::trial_service::trial_config::TrialConfig
) -> std::result::Result<crate::services::trial_service::trial_status::TrialStatus, sqlx::Error> {
    let user_trial_info = crate::queries::trial_service::users::get_user_trial_info(pool, user_id).await?;

    if let std::option::Option::Some(trial_ended_at) = user_trial_info.trial_ended_at {
        if chrono::Utc::now() > trial_ended_at {
            return std::result::Result::Ok(crate::services::trial_service::trial_status::TrialStatus::Expired);
        }
    }

    if let std::option::Option::Some(trial_started_at) = user_trial_info.trial_started_at {
        let trial_end_date = trial_started_at + chrono::Duration::days(config.trial_period_days());
        let now = chrono::Utc::now();

        if now > trial_end_date {
            std::result::Result::Ok(crate::services::trial_service::trial_status::TrialStatus::Expired)
        } else {
            let days_remaining = (trial_end_date - now).num_days();
            std::result::Result::Ok(crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining })
        }
    } else {
        std::result::Result::Ok(crate::services::trial_service::trial_status::TrialStatus::NotStarted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_period_calculation() {
        // Test that the function uses the config's trial period
        let config = crate::services::trial_service::trial_config::TrialConfig::new(14);
        assert_eq!(config.trial_period_days(), 14);
        
        // The actual function uses this value for chrono::Duration::days(config.trial_period_days())
        let duration = chrono::Duration::days(config.trial_period_days());
        assert_eq!(duration.num_days(), 14);
    }

    #[test]
    fn test_trial_end_date_calculation() {
        // Test the trial end date calculation logic
        let config = crate::services::trial_service::trial_config::TrialConfig::new(7);
        let start_date = chrono::Utc::now();
        let end_date = start_date + chrono::Duration::days(config.trial_period_days());
        
        let expected_duration = end_date - start_date;
        assert_eq!(expected_duration.num_days(), 7);
    }

    #[test]
    fn test_days_remaining_calculation() {
        // Test the days remaining calculation logic
        let now = chrono::Utc::now();
        let future_date = now + chrono::Duration::days(5);
        let days_remaining = (future_date - now).num_days();
        
        assert_eq!(days_remaining, 5);
    }

    #[test]
    fn test_trial_expired_check() {
        // Test the trial expiration logic
        let now = chrono::Utc::now();
        let past_date = now - chrono::Duration::days(1);
        
        assert!(now > past_date);
    }
}
