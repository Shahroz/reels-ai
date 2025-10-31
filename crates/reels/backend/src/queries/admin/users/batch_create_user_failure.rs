//! Failure result for batch user creation.
//!
//! Contains the email and failure reason for a user that could not be
//! created in a batch operation (validation error, already exists, etc.).
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from batch_create_users.rs

pub struct BatchCreateUserFailure {
    pub email: std::string::String,
    pub reason: std::string::String,
}

