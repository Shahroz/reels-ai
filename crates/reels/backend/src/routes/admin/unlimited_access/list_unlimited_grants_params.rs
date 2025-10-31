//! Query parameters for listing unlimited access grants.
//!
//! This struct defines the query parameters accepted by the admin endpoint
//! for listing unlimited access grants. Supports pagination and filtering
//! by active/revoked status.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query parameters for listing unlimited access grants
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ListUnlimitedGrantsParams {
    #[param(example = 20)]
    pub limit: Option<i64>,
    
    #[param(example = 0)]
    pub offset: Option<i64>,
    
    #[param(example = false)]
    pub include_revoked: Option<bool>,
}

