//! Response body for creating a vocal tour.
//!
//! Defines the `CreateVocalTourResponse` structure with the created document and assets.

use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateVocalTourResponse {
    /// The id of the
    pub vocal_tour_id: Uuid,
    /// The newly created document containing the property description.
    pub document: crate::db::documents::Document,
    /// An array of newly created assets from the tour.
    pub created_assets: std::vec::Vec<crate::db::assets::Asset>,
    /// Total number of assets created from the vocal tour.
    #[schema(example = 25)]
    pub total_assets: usize,
} 