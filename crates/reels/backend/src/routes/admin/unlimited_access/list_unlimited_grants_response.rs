//! Response schema for listing unlimited access grants.
//!
//! This struct defines the response structure returned when listing
//! unlimited access grants. Includes array of grants and total count.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response with list of unlimited access grants
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListUnlimitedGrantsResponse {
    pub grants: Vec<crate::db::unlimited_access_grant::UnlimitedAccessGrant>,
    pub total: usize,
}

