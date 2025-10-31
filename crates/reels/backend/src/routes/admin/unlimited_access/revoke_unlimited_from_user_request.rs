//! Request schema for revoking unlimited access from a user.
//!
//! This struct defines the payload structure for the admin endpoint
//! that revokes unlimited credit access from a user. Requires a reason
//! for the revocation for audit trail purposes.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to revoke unlimited access from a user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RevokeUnlimitedFromUserRequest {
    #[schema(example = "Trial period ended")]
    pub reason: String,
}

