//! Defines the request body for creating a one-time research task.
//!
//! This struct captures the necessary information from the user to initiate
//! a new one-time research task.

use serde::Deserialize;
use utoipa::ToSchema;

/// Request payload for creating a new one-time research task.
#[derive(Deserialize, ToSchema)]
pub struct CreateOneTimeResearchRequest {
    /// The detailed prompt or instruction for the research task.
    #[schema(example = "Research the top 5 AI startups in Europe and their latest funding rounds.")]
    pub prompt: String,
    
    /// Optional organization ID to deduct credits from (if user is acting on behalf of an organization)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = Option<String>)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}