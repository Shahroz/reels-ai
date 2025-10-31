//! Trial service module exports following one-item-per-file pattern.
//!
//! This module contains all trial and billing status related functionality split
//! into individual files for better modularity and graph-based code structure.
//! Each file contains exactly one logical item (function, struct, or enum) with
//! comprehensive unit tests and proper documentation.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Converted to modular structure during file splitting
//! - [Prior updates not documented in original file]

pub mod trial_config;
pub mod get_trial_period_days;
pub mod trial_status;
pub mod billing_status;
pub mod get_trial_status;
pub mod get_trial_status_with_config;
pub mod get_billing_status;
pub mod get_billing_status_with_config;
pub mod has_organization_access;
pub mod has_access;
pub mod has_access_with_config;
pub mod is_trial_expired;
pub mod end_trial;
pub mod activate_subscription;

// Re-exports for backward compatibility
pub use trial_config::TrialConfig;
pub use get_trial_period_days::get_trial_period_days;
pub use trial_status::TrialStatus;
pub use billing_status::BillingStatus;
pub use get_trial_status::get_trial_status;
pub use get_trial_status_with_config::get_trial_status_with_config;
pub use get_billing_status::get_billing_status;
pub use get_billing_status_with_config::get_billing_status_with_config;
#[allow(deprecated)]
pub use has_organization_access::has_organization_access; // DEPRECATED as of 2025-10-17
pub use has_access::has_access;
pub use has_access_with_config::has_access_with_config;
pub use is_trial_expired::is_trial_expired;
pub use end_trial::end_trial;
pub use activate_subscription::activate_subscription;
