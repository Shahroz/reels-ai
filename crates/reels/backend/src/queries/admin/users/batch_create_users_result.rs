//! Result structure for batch user creation.
//!
//! Aggregates all successes and failures from a batch user creation
//! operation to support partial success scenarios in admin bulk operations.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from batch_create_users.rs

pub struct BatchCreateUsersResult {
    pub success: std::vec::Vec<crate::queries::admin::users::batch_create_user_success::BatchCreateUserSuccess>,
    pub failed: std::vec::Vec<crate::queries::admin::users::batch_create_user_failure::BatchCreateUserFailure>,
}


