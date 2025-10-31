//! Defines the request payload for generating a creative via LLM.
//!
//! Contains IDs referencing the necessary components like style, assets,
//! optional document, and creative formats.
//! Adheres to one-item-per-file guideline.
//! Uses fully qualified paths and non-FQP in annotations.

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct GenerateCreativeRequest {
    /// Required name for the creative.
    #[schema(example = "My Creative", value_type = String)]
    pub name: std::string::String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440005", format = "uuid", value_type=String)]
    pub collection_id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=Option<String>, nullable = true)]
    pub style_id: Option<uuid::Uuid>,
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440001", "550e8400-e29b-41d4-a716-446655440002"]), value_type=Option<Vec<String>>)]
    pub asset_ids: Option<std::vec::Vec<uuid::Uuid>>, // Changed to Option<Vec<Uuid>>
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440003"]), format = "uuid", value_type=Option<Vec<String>>)]
    pub document_ids: Option<std::vec::Vec<uuid::Uuid>>, // Renamed and changed to Option<Vec<Uuid>>
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440004"]), format = "uuid", value_type=Vec<String>)]
    pub creative_format_ids: std::vec::Vec<uuid::Uuid>, // Renamed and changed to Vec<Uuid> (non-optional)
    #[schema(example = json!(["bundle-uuid-1", "bundle-uuid-2"]), value_type = Option<Vec<String>>, nullable = true)]
    pub bundle_ids: Option<std::vec::Vec<uuid::Uuid>>,
    
    /// Optional organization ID to deduct credits from (if user is acting on behalf of an organization)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = Option<String>)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}