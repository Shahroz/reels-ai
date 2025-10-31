//! Defines the request structure for creating a new API key.
//!
//! This struct holds the user_id and optional allowed_domains for API key creation.
//! It is deserialized from JSON for the API request.
//! Conforms to the coding standards by being the sole item in this file.
//! Uses `serde` for deserialization and `utoipa` for schema generation.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request structure for creating a new API key.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    /// The ID of the user to create the API key for.
    /// Required for admin users creating keys for other users.
    /// Optional for non-admin users (will use their own user_id).
    #[schema(value_type = Option<String>)] // Correct Uuid representation for OpenAPI
    pub user_id: Option<Uuid>,
    /// Comma-separated list of allowed domains for API key usage. 
    /// If None or empty, no domain restrictions are applied.
    pub allowed_domains: Option<String>,
}
