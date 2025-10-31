//! Request body for creating a vocal tour.
//!
//! Defines the `CreateVocalTourRequest` structure with asset IDs to be processed.

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateVocalTourRequest {
    /// The UUIDs of the initially uploaded assets (videos, photos) to be processed.
    #[schema(example = r#"["550e8400-e29b-41d4-a716-446655440000", "550e8400-e29b-41d4-a716-446655440001"]"#)]
    pub asset_ids: std::vec::Vec<std::string::String>,
    /// Optional collection ID to attach the document and assets to.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid", value_type = Option<String>, nullable = true)]
    pub collection_id: std::option::Option<std::string::String>,
} 