//! BillingStatus struct representing comprehensive billing information for a user.
//!
//! This structure combines trial status, subscription details, and payment information
//! to provide a complete view of a user's billing state. Used by billing endpoints
//! and access control logic to make informed decisions about user permissions.
//! The structure includes Stripe customer ID for payment integration capabilities.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize)]
pub struct BillingStatus {
    pub trial_status: crate::services::trial_service::trial_status::TrialStatus,
    pub subscription_status: std::string::String,
    pub has_active_subscription: bool,
    pub stripe_customer_id: std::option::Option<std::string::String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_billing_status_creation() {
        let billing_status = BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining: 5 },
            subscription_status: "trial".to_string(),
            has_active_subscription: false,
            stripe_customer_id: std::option::Option::None,
        };

        assert!(matches!(billing_status.trial_status, crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining: 5 }));
        assert_eq!(billing_status.subscription_status, "trial");
        assert!(!billing_status.has_active_subscription);
        assert!(billing_status.stripe_customer_id.is_none());
    }

    #[test]
    fn test_billing_status_with_subscription() {
        let billing_status = BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::Expired,
            subscription_status: "active".to_string(),
            has_active_subscription: true,
            stripe_customer_id: std::option::Option::Some("cus_test123".to_string()),
        };

        assert!(matches!(billing_status.trial_status, crate::services::trial_service::trial_status::TrialStatus::Expired));
        assert_eq!(billing_status.subscription_status, "active");
        assert!(billing_status.has_active_subscription);
        assert_eq!(billing_status.stripe_customer_id.unwrap(), "cus_test123");
    }

    #[test]
    fn test_billing_status_serialization() {
        let billing_status = BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::NotStarted,
            subscription_status: "none".to_string(),
            has_active_subscription: false,
            stripe_customer_id: std::option::Option::None,
        };

        let serialized = serde_json::to_string(&billing_status).unwrap();
        let deserialized: BillingStatus = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(deserialized.trial_status, crate::services::trial_service::trial_status::TrialStatus::NotStarted));
        assert_eq!(deserialized.subscription_status, "none");
        assert!(!deserialized.has_active_subscription);
        assert!(deserialized.stripe_customer_id.is_none());
    }

    #[test]
    fn test_billing_status_clone() {
        let original = BillingStatus {
            trial_status: crate::services::trial_service::trial_status::TrialStatus::Active { days_remaining: 7 },
            subscription_status: "trial".to_string(),
            has_active_subscription: false,
            stripe_customer_id: std::option::Option::Some("cus_clone_test".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(cloned.subscription_status, original.subscription_status);
        assert_eq!(cloned.has_active_subscription, original.has_active_subscription);
        assert_eq!(cloned.stripe_customer_id, original.stripe_customer_id);
    }
}
