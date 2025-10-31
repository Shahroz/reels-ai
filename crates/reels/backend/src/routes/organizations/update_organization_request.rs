use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, serde::Serialize)]
pub struct UpdateOrganizationRequest {
    #[schema(example = "New Organization Name")]
    pub name: Option<String>,
    // Add other updatable fields here later, e.g., settings
} 