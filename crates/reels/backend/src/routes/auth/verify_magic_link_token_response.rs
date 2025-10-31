//! Response body for successful magic link token verification.
//!
//! Returns session token and user data after successful magic link
//! authentication via the POST API endpoint.

/// Response body for successful magic link verification.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct VerifyMagicLinkTokenResponse {
    /// Long-lived session JWT token (30 days)
    pub session_token: std::string::String,
    /// User's public profile data
    pub user: crate::db::users::PublicUser,
}

