//! Defines the structured responses for document-related API endpoints.
use crate::db::documents::Document;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a document returned by the API, including associated metadata.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DocumentResponse {
    #[serde(flatten)]
    #[schema(inline)]
    pub document: Document,
    #[schema(example = "user@example.com")]
    pub creator_email: Option<String>,
    #[schema(example = "owner")]
    pub current_user_access_level: Option<String>,
}

/// Represents a document returned by the API, including associated metadata.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DocumentResponseWithFavorite {
    #[serde(flatten)]
    #[schema(inline)]
    pub document: Document,
    #[schema(example = "user@example.com")]
    pub creator_email: Option<String>,
    #[schema(example = "owner")]
    pub current_user_access_level: Option<String>,
    #[schema(example = false)]
    pub is_favorite: bool,
}