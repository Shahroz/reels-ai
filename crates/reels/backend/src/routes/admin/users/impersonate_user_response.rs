//! Response struct for successful user impersonation.
//!
//! Contains the new JWT token for the impersonated session and public user information
//! of the impersonated user.

use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

/// Response payload for a successful user impersonation.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImpersonateUserResponse {
    /// The new JWT token for the impersonation session.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: std::string::String,
    /// Public details of the impersonated user.
    #[schema(value_type = crate::db::users::PublicUser)]
    pub user: crate::db::users::PublicUser,
}