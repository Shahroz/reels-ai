//! Success result for batch user creation.
//!
//! Contains the email and newly created user details for a successfully
//! created user in a batch operation.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from batch_create_users.rs

pub struct BatchCreateUserSuccess {
    pub email: std::string::String,
    pub user: crate::db::users::PublicUser,
}


