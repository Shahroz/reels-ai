use crate::db;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a creative in an API response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreativeResponse {
    #[serde(flatten)]
    #[schema(inline)]
    pub creative: db::creatives::Creative,
    #[schema(example = "user@example.com", nullable = true)]
    pub creator_email: Option<String>,
    /// The current authenticated user's access level to this creative (e.g., owner, editor, viewer).
    #[schema(example = "editor", nullable = true)]
    pub current_user_access_level: Option<String>,
}

/// Represents a creative with all related data for detailed view.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetCreativeDetails {
    #[serde(flatten)]
    #[schema(inline)]
    pub creative: db::creatives::Creative,
    #[schema(example = "user@example.com", nullable = true)]
    pub creator_email: Option<String>,
    /// The current authenticated user's access level to this creative (e.g., owner, editor, viewer).
    #[schema(example = "editor", nullable = true)]
    pub current_user_access_level: Option<String>,
    /// Whether the current user has marked this creative as a favorite.
    #[schema(example = true)]
    pub is_favorite: bool,
    /// The style associated with this creative, if any.
    #[schema(nullable = true)]
    pub style: Option<db::styles::Style>,
    /// The assets associated with this creative, if any.
    #[schema(nullable = true)]
    pub assets: Vec<db::assets::Asset>,
    /// The documents associated with this creative, if any.
    #[schema(nullable = true)]
    pub documents: Vec<db::documents::Document>,
    /// The creative format used for this creative.
    #[schema(nullable = true)]
    pub creative_format: Option<db::custom_creative_formats::CustomCreativeFormat>,
    /// The collection this creative belongs to, if any.
    #[schema(nullable = true)]
    pub collection: Option<db::collections::Collection>,
    /// The bundle this creative belongs to, if any.
    #[schema(nullable = true)]
    pub bundle: Option<db::bundles::Bundle>,
} 