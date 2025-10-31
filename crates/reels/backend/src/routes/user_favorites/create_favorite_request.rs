//! Defines the request body for creating a user favorite.
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateFavoriteRequest {
    #[schema(example = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")]
    pub entity_id: Uuid,
    #[schema(example = "creative")]
    pub entity_type: String, // e.g., "style", "creative", "document"
} 