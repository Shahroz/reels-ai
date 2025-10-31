//! Defines the request body for the password reset endpoint.
//!
//! This struct captures the necessary information provided by a user
//! to reset their password: a valid token and their new desired password.

/// The request payload for resetting a user's password.
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    /// The password reset token sent to the user's email.
    #[schema(example = "a1b2c3d4...", value_type = String)]
    pub token: String,
    /// The user's new password.
    #[schema(example = "newSecurePassword123!", value_type = String)]
    pub password: String,
}