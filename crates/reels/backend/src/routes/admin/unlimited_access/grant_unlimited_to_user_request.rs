//! Request schema for granting unlimited access to a user.
//!
//! This struct defines the payload structure for the admin endpoint
//! that grants unlimited credit access to a user. It includes the reason
//! for granting access, optional expiration date, and optional notes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to grant unlimited access to a user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GrantUnlimitedToUserRequest {
    #[schema(example = "Early adopter grandfather clause")]
    pub reason: String,
    
    #[schema(value_type = String, format = "date-time", example = "2024-12-31T23:59:59Z")]
    pub expires_at: Option<DateTime<Utc>>,
    
    #[schema(example = "VIP customer - unlimited until end of 2024")]
    pub notes: Option<String>,
}

