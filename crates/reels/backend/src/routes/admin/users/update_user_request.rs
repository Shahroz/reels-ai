//! Defines the request body for updating a user via the admin panel.
//!
//! This structure captures all necessary details for an administrator
//! to update a user account with specific attributes.

use serde::Deserialize;
use utoipa::ToSchema;

/// The request payload for an admin to update a user.
#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// Flag indicating if the user should have administrative privileges.
    #[schema(example = false)]
    pub is_admin: bool,

    /// The new status for the user account.
    #[schema(example = "active")]
    pub status: String,

    /// A list of feature flags to be enabled for the user.
    #[schema(example = json!(["beta_access", "early_features"]))]
    pub feature_flags: Vec<String>,
}