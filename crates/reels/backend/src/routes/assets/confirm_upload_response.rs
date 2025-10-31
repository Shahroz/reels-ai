//! Response structure for confirming asset uploads.
//!
//! Defines the structure returned when an asset upload has been
//! successfully confirmed and registered in the database.

use crate::db::assets::Asset;

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ConfirmUploadResponse {
    pub asset: Asset,
    #[schema(example = "confirmed")]
    pub status: std::string::String,
    #[schema(example = "Asset successfully registered")]
    pub message: std::string::String,
} 