//! Response struct for successful token verification.
//!
//! Contains the public user information associated with the valid token.

use serde::Serialize;
use utoipa::ToSchema;
use crate::db::users::PublicUser;

/// Represents the successful response for the token verification endpoint.
#[derive(Serialize, ToSchema)]
pub struct VerifyTokenResponse {
    /// Public details of the user associated with the token.
    #[schema(value_type = PublicUser)]
    pub user: crate::db::users::PublicUser,
    /// Indicates if the current session is an impersonation session.
    #[schema(example = false)]
    pub is_impersonating: bool,
    /// If impersonating, contains the public details of the original administrator.
    #[schema(nullable = true)]
    pub original_admin_user: Option<PublicUser>,
}
