//! Response DTO for batch deleting users.
//!
//! This DTO provides detailed results for a batch user deletion operation,
//! including both successful and failed deletions. Uses 207 Multi-Status
//! pattern to support partial success scenarios with safety guardrails.

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct BatchDeleteUsersResponse {
    /// Successfully deleted users
    pub success: Vec<UserDeleteSuccess>,
    /// Failed deletion attempts
    pub failed: Vec<UserDeleteFailure>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserDeleteSuccess {
    /// ID of deleted user
    pub user_id: uuid::Uuid,
    /// Email of deleted user
    pub email: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserDeleteFailure {
    /// ID of user that failed to delete
    pub user_id: uuid::Uuid,
    /// Reason for failure
    pub reason: String,
}
