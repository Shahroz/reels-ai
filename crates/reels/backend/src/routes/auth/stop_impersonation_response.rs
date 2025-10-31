//! Response struct for successfully stopping a user impersonation session.
//!
//! Contains the new JWT token for the original admin, public user information
//! of the admin, and a confirmation message.

use crate::db::users::PublicUser;
use serde::Serialize;
use utoipa::ToSchema;

/// Response payload for stopping a user impersonation session.
#[derive(Serialize, ToSchema)]
pub struct StopImpersonationResponse {
    /// The new JWT token for the original administrator's session.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// Public details of the original administrator.
    #[schema(value_type = PublicUser)]
    pub user: PublicUser,
    /// A message indicating that impersonation has stopped.
    #[schema(example = "Impersonation stopped. Returned to admin session.")]
    pub message: String,
}