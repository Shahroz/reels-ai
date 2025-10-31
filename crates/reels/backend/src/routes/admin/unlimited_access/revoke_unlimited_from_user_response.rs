//! Response schema for revoking unlimited access from a user.
//!
//! This struct defines the response structure returned after successfully
//! revoking unlimited access from a user. Includes the revoked grant details
//! and a success message.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response after revoking unlimited access from a user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RevokeUnlimitedFromUserResponse {
    pub grant: crate::db::unlimited_access_grant::UnlimitedAccessGrant,
    pub message: String,
}

