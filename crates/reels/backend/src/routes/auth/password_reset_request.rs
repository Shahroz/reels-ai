//! Request struct for password reset initiation.
//!
//! Defines the expected JSON payload for initiating a password reset.
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct PasswordResetRequest {
    pub email: std::string::String,
}
