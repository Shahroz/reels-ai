//! Request struct for user login.
//!
//! Defines the expected JSON payload for logging in.
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: std::string::String,
    pub password: std::string::String,
}
