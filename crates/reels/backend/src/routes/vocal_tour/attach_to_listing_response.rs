//! Response model for attaching vocal tour content to a listing.
//!
//! This struct defines the response body for the endpoint that attaches assets
//! and documents from a vocal tour to a collection/listing. Returns the count of 
//! successfully attached items and the target collection ID for confirmation.
//! Note: attached_asset_count may be 0 if the vocal tour has no assets.

#[derive(serde::Serialize, utoipa::ToSchema, std::fmt::Debug)]
pub struct AttachVocalTourToListingResponse {
    #[schema(example = 0)]
    pub attached_asset_count: usize,
    #[schema(example = 3)]
    pub attached_document_count: usize,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub collection_id: uuid::Uuid,
} 