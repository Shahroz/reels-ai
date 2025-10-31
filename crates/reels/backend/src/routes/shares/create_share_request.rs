//! Defines the request body for creating or updating an object share.
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CreateShareRequest {
    #[schema(example = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")]
    pub object_id: Uuid,
    #[schema(example = "style")]
    pub object_type: String, // e.g., "style", "creative", "research"
    
    #[schema(example = "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy", nullable = true)]
    pub entity_id: Option<Uuid>, // User ID or Organization ID, now optional
    
    #[schema(example = "user@example.com", nullable = true)]
    pub entity_email: Option<String>, // Optional email for user shares

    #[schema(example = "user")]
    pub entity_type: String, // "user" or "organization"
    #[schema(example = "viewer")]
    pub access_level: String, // e.g., "viewer", "editor"
} 