//! Request body for enhancing one or more existing assets with AI.
//!
//! Defines the `EnhanceAssetRequest` with one or more asset IDs and an optional retouch prompt.

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct EnhanceAssetRequest {
    /// The UUIDs of the assets to enhance
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440000", "beefcafe-e29b-41d4-a716-446655440000"]))]
    pub asset_ids: std::vec::Vec<std::string::String>,
    /// Optional prompt to guide the AI enhancement process
    #[schema(example = "Enhance lighting and remove blemishes")]
    pub retouch_prompt: std::option::Option<std::string::String>,

    /// Optional ID of the vocal tour to associate the enhanced assets with.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = Option<String>)]
    pub vocal_tour_id: std::option::Option<std::string::String>,

    /// Optional flag to indicate this is a regenerate action (for analytics)
    #[schema(example = false)]
    #[serde(default)]
    pub is_regenerate: bool,
}
