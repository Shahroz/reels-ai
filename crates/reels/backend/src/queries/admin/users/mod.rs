//! Admin user query operations.
//!
//! This module exposes admin-level queries for managing users in batch. These queries
//! power bulk user creation, deletion, and organization membership management.

pub mod batch_create_users;
pub mod batch_create_user_success;
pub mod batch_create_user_failure;
pub mod batch_create_users_result;
pub mod is_valid_email;
pub mod batch_delete_users;
pub mod find_users_by_emails;
pub mod list_users_with_credits;
pub mod enriched_user;
pub mod services;

pub use batch_create_users::batch_create_users;
pub use batch_delete_users::batch_delete_users;
pub use find_users_by_emails::find_users_by_emails;
pub use list_users_with_credits::list_users_with_credits;
pub use enriched_user::EnrichedUser;
