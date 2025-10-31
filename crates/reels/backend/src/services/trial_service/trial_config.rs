//! TrialConfig struct for managing trial configuration with testable environment dependencies.
//!
//! This structure encapsulates trial-related configuration parameters, providing explicit
//! dependency injection for environment variables rather than direct environment access.
//! Enables deterministic testing by allowing tests to use known configuration values
//! instead of relying on external environment state. Supports both environment-based
//! initialization for production use and explicit initialization for testing.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created for environment dependency optimization
//! - [Prior updates not documented in original file]

#[derive(std::fmt::Debug, std::clone::Clone)]
pub struct TrialConfig {
    trial_period_days: i64,
}

impl TrialConfig {
    pub fn from_env() -> Self {
        Self {
            trial_period_days: std::env::var("STRIPE_TRIAL_PERIOD_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7)
        }
    }
    
    pub fn new(trial_period_days: i64) -> Self {
        Self { trial_period_days }
    }
    
    pub fn trial_period_days(&self) -> i64 {
        self.trial_period_days
    }
}

impl std::default::Default for TrialConfig {
    fn default() -> Self {
        Self::new(7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_config_new() {
        let config = TrialConfig::new(14);
        assert_eq!(config.trial_period_days(), 14);
    }

    #[test]
    fn test_trial_config_default() {
        let config = TrialConfig::default();
        assert_eq!(config.trial_period_days(), 7);
    }

    #[test]
    fn test_trial_config_clone() {
        let original = TrialConfig::new(30);
        let cloned = original.clone();
        assert_eq!(cloned.trial_period_days(), 30);
    }

    #[test]
    fn test_trial_config_debug() {
        let config = TrialConfig::new(21);
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("21"));
    }

    #[test]
    fn test_trial_config_from_env_fallback() {
        // Test that from_env works even without environment variable
        // (this will use the fallback value of 7)
        let config = TrialConfig::from_env();
        let days = config.trial_period_days();
        assert!(days > 0);
        assert!(days <= 365);
    }

    #[test]
    fn test_trial_config_reasonable_bounds() {
        let config = TrialConfig::new(10);
        let days = config.trial_period_days();
        assert!(days > 0, "Trial period should be positive");
        assert!(days <= 365, "Trial period should be reasonable");
    }
}
