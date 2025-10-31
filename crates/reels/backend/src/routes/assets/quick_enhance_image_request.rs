//! Request structure for Quick Enhance Image API.
//!
//! This module defines the request structure for the `/api/assets/quick-enhance-image` endpoint.
//! It contains the image data (base64 encoded) and enhancement prompt required for processing.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request structure for Quick Enhance Image.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuickEnhanceImageRequest {
    /// Base64 encoded image data to be enhanced (alternative to asset_id)
    /// Can be either raw base64 data or a data URL (data:image/type;base64,...)
    #[schema(example = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD...")]
    pub image_data: Option<String>,

    /// Asset ID to fetch image from GCS (alternative to image_data)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub asset_id: Option<uuid::Uuid>,

    /// Enhancement prompt describing the desired modifications
    #[schema(example = "Enhance the lighting, remove blemishes, and improve overall quality")]
    pub enhancement_prompt: String,

    /// Optional MIME type for the enhanced image output (e.g., "image/jpeg", "image/png")
    #[schema(example = "image/jpeg")]
    pub output_mime_type: Option<String>,
}
