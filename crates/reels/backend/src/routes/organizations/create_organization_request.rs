//! Defines the request payload for creating a new organization.
//!
//! This structure captures the necessary information, like the organization's name,
//! required to create a new organization entry via the API.
//! Adheres to project guidelines, including deriving necessary traits.

use serde::Deserialize;
use utoipa::ToSchema;

/// Payload for creating a new organization.
#[derive(Debug, Clone, Deserialize, ToSchema, serde::Serialize)]
pub struct CreateOrganizationRequest {
    /// The desired name for the new organization.
    #[schema(example = "New Ventures Inc.")]
    pub name: String,
}
