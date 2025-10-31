//! Response struct for a standard logout operation.

use serde::Serialize;
use utoipa::ToSchema;

/// Response payload for a standard user logout.
#[derive(Serialize, ToSchema)]
pub struct StandardLogoutResponse {
    /// A message indicating the outcome of the logout operation.
    #[schema(example = "Logout successful.")]
    pub message: String,
}