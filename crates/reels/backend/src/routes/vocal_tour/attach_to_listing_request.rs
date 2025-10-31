//! Request model for attaching vocal tour assets to a listing.
//!
//! This struct defines the request body for the endpoint that attaches all assets
//! from a vocal tour to a collection/listing. Contains the target collection ID
//! where the vocal tour assets should be attached.

#[derive(serde::Deserialize, utoipa::ToSchema, std::fmt::Debug)]
pub struct AttachVocalTourToListingRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub collection_id: uuid::Uuid,
} 