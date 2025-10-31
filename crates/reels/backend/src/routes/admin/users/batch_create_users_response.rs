//! Response DTO for batch creating users.
//!
//! This DTO provides detailed results for a batch user creation operation,
//! including both successful and failed user creations. Uses 207 Multi-Status
//! pattern to support partial success scenarios.

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct BatchCreateUsersResponse {
    /// Successfully created users
    pub success: Vec<UserCreateSuccess>,
    /// Failed user creation attempts
    pub failed: Vec<UserCreateFailure>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserCreateSuccess {
    /// Email address of created user
    pub email: String,
    /// The created user record
    pub user: crate::db::users::PublicUser,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserCreateFailure {
    /// Email address that failed
    pub email: String,
    /// Reason for failure
    pub reason: String,
}
