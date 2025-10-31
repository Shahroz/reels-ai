//! Enum to represent the possible response bodies for the logout endpoint.
//!
//! This is used for OpenAPI documentation to show that the logout endpoint
//! can return different structures based on whether an impersonation session
//! is being stopped or a standard logout is occurring.

use crate::routes::auth::standard_logout_response::StandardLogoutResponse;
use crate::routes::auth::stop_impersonation_response::StopImpersonationResponse;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(untagged)] // Ensures correct JSON serialization based on the variant
pub enum LogoutResponseBody {
    /// Response for a standard logout.
    Standard(StandardLogoutResponse),
    /// Response for stopping an impersonation session.
    ImpersonationStopped(StopImpersonationResponse),
}