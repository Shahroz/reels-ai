//! Defines the response body for the public journey view endpoint.

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicJourneyNodeResponse {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=String)]
    pub id: Uuid,
    #[schema(example = "Initial Image")]
    pub name: String,
    #[schema(example = "https://path/to/image.png", format = "uri")]
    pub url: String,
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=Option<String>, nullable = true)]
    pub parent_id: Option<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct GetPublicJourneyResponse {
    #[schema(example = "My Awesome Journey")]
    pub name: String,
    pub nodes: Vec<PublicJourneyNodeResponse>,
}