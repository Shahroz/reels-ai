//! Module for user subscription related database query functions.
//!
//! This module aggregates specific query operations for user subscription entities.
//! It adheres to the one-item-per-file and FQN guidelines.
//! Organizes functions for user subscription management.

pub mod create_user_subscription;
pub mod get_user_subscription_by_user_id;
pub mod get_user_subscription_by_stripe_id;
pub mod get_user_subscription_by_stripe_price_id;
pub mod get_current_active_subscription;
pub mod get_subscriptions_by_status;
pub mod update_user_subscription_status;
pub mod update_user_subscription_by_user_id;
pub mod delete_user_subscription;
pub mod delete_user_subscription_by_user_id;
pub mod cancel_all_subscriptions_except;

// Re-export all functions for convenience
pub use create_user_subscription::create_user_subscription;
pub use get_user_subscription_by_user_id::get_user_subscription_by_user_id;
pub use get_user_subscription_by_stripe_id::get_user_subscription_by_stripe_id;
pub use get_user_subscription_by_stripe_price_id::get_user_subscription_by_stripe_price_id;
pub use get_current_active_subscription::get_current_active_subscription;
pub use get_subscriptions_by_status::get_subscriptions_by_status;
pub use update_user_subscription_status::update_user_subscription_status;
pub use update_user_subscription_by_user_id::update_user_subscription_by_user_id;
pub use delete_user_subscription::delete_user_subscription;
pub use delete_user_subscription_by_user_id::delete_user_subscription_by_user_id;
pub use cancel_all_subscriptions_except::cancel_all_subscriptions_except;
