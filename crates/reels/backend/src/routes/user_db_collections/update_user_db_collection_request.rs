//! Defines the request body for updating an existing user DB collection.
//!
//! This struct allows for partial updates to a collection's metadata,
//! specifically its name and description. Fields set to `None` will not be updated.
//! To clear the description, provide `Some(None)`.
//! Adheres to 'one item per file' and FQN guidelines.

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserDbCollectionRequest {
    #[schema(example = "My Updated Photo Album")]
    pub name: Option<String>,
    #[schema(example = "An updated collection of all holiday photos including 2024.")]
    pub description: Option<Option<String>>, // Option<Option<String>> to allow setting description to NULL
}
