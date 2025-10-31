//! Configuration function for retrieving trial period duration in days.
//!
//! This function provides configurable trial period duration via the STRIPE_TRIAL_PERIOD_DAYS
//! environment variable with a default fallback of 7 days. Used throughout the billing system
//! to ensure consistent trial period calculations. Environment variable approach allows
//! runtime configuration without code changes for different deployment environments.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

pub fn get_trial_period_days() -> i64 {
    std::env::var("STRIPE_TRIAL_PERIOD_DAYS")
        .unwrap_or_else(|_| "7".to_string())
        .parse()
        .unwrap_or(7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_trial_period_days_default() {
        // Test default value when environment variable is not set
        let trial_days = get_trial_period_days();
        assert!(trial_days > 0);
        assert!(trial_days <= 365);
    }

    #[test]
    fn test_get_trial_period_days_reasonable_bounds() {
        // Ensure the value is within reasonable business bounds
        let trial_days = get_trial_period_days();
        assert!(trial_days >= 1, "Trial period should be at least 1 day");
        assert!(trial_days <= 365, "Trial period should not exceed 1 year");
    }
}
