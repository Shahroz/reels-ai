//! Defines the request payload for updating a creative's name.
//!
//! This struct is used to pass the new name for an existing creative.
//! It contains a single field, `name`, which specifies the desired new name.
//! The request is typically deserialized from JSON and handled by an endpoint like `update_creative_name`.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateCreativeNameRequest {
    #[schema(example = "Updated Creative Name")]
    pub name: std::string::String,
} 