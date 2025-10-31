//! Request struct for changing a user's password.
//!
//! Defines the expected JSON payload for the change password endpoint.
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    /// The user's current password.
    #[schema(example = "currentSecurePassword123")]
    pub current_password: String,
    /// The new password the user wants to set.
    #[schema(example = "newSuperSecurePassword456")]
    pub new_password: String,
}