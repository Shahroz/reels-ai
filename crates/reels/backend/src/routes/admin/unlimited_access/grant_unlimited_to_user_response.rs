//! Response schema for granting unlimited access to a user.
//!
//! This struct defines the response structure returned after successfully
//! granting unlimited access to a user. Includes the created grant details
//! and a success message.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response after granting unlimited access to a user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GrantUnlimitedToUserResponse {
    pub grant: crate::db::unlimited_access_grant::UnlimitedAccessGrant,
    pub message: String,
}

