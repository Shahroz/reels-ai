//! Defines the request body for creating a user via the admin panel.
//!
//! This structure captures all necessary details for an administrator
//! to create a new user account with specific attributes.

use serde::Deserialize;
use utoipa::ToSchema;

/// The request payload for an admin to create a new user.
#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// The email address for the new user.
    #[schema(example = "new.user@example.com")]
    pub email: String,

    /// The plaintext password for the new user. This will be hashed before storage.
    #[schema(example = "a-very-secure-password!")]
    pub password: String,

    /// Flag indicating if the new user should have administrative privileges.
    #[schema(example = false)]
    pub is_admin: bool,

    /// The initial status for the new user account.
    #[schema(example = "active")]
    pub status: String,

    /// A list of feature flags to be enabled for the new user.
    #[schema(example = json!(["beta_access", "early_features"]))]
    pub feature_flags: Vec<String>,
}