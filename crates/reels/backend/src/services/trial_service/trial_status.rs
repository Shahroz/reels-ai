//! TrialStatus enum representing the current state of a user's trial period.
//!
//! This enum encapsulates the three possible states of a user trial: active with remaining days,
//! expired, or not yet started. The Active variant includes the number of days remaining
//! to provide precise trial expiration information. Used throughout the billing system
//! to make trial-related access decisions and display appropriate user interface elements.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize)]
pub enum TrialStatus {
    Active { days_remaining: i64 },
    Expired,
    NotStarted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_status_serialization_active() {
        let status = TrialStatus::Active { days_remaining: 5 };
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: TrialStatus = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            TrialStatus::Active { days_remaining } => assert_eq!(days_remaining, 5),
            _ => panic!("Expected Active status"),
        }
    }

    #[test]
    fn test_trial_status_serialization_expired() {
        let status = TrialStatus::Expired;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: TrialStatus = serde_json::from_str(&serialized).unwrap();
        
        assert!(matches!(deserialized, TrialStatus::Expired));
    }

    #[test]
    fn test_trial_status_serialization_not_started() {
        let status = TrialStatus::NotStarted;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: TrialStatus = serde_json::from_str(&serialized).unwrap();
        
        assert!(matches!(deserialized, TrialStatus::NotStarted));
    }

    #[test]
    fn test_trial_status_clone() {
        let original = TrialStatus::Active { days_remaining: 10 };
        let cloned = original.clone();
        
        match (original, cloned) {
            (TrialStatus::Active { days_remaining: orig }, TrialStatus::Active { days_remaining: clone }) => {
                assert_eq!(orig, clone);
            }
            _ => panic!("Clone should preserve the same variant and values"),
        }
    }

    #[test]
    fn test_trial_status_debug_format() {
        let status = TrialStatus::Active { days_remaining: 3 };
        let debug_string = format!("{:?}", status);
        assert!(debug_string.contains("Active"));
        assert!(debug_string.contains("3"));
    }
}
