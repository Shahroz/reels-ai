//! Request body for magic link token verification API endpoint.
//!
//! Used by the POST /auth/verify-magic-link-token endpoint to receive
//! the JWT token for API-based magic link authentication.

/// Request body for magic link token verification.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct VerifyMagicLinkTokenRequest {
    /// The magic link JWT token from the email
    pub token: std::string::String,
}

