//! Response structure for Quick Enhance Image API.
//!
//! This module defines the response structure for the `/api/assets/quick-enhance-image` endpoint.
//! It contains the enhanced image data and metadata about the processing.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response structure for Quick Enhance Image.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuickEnhanceImageResponse {
    /// Base64 encoded enhanced image data
    #[schema(example = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD...")]
    pub enhanced_image_data: String,

    /// Original enhancement prompt that was used
    #[schema(example = "Enhance the lighting, remove blemishes, and improve overall quality")]
    pub original_prompt: String,

    /// Processing time in milliseconds
    #[schema(example = 1250)]
    pub processing_time_ms: u64,

    /// Whether the enhancement was successful
    #[schema(example = true)]
    pub enhancement_successful: bool,

    /// MIME type of the enhanced image
    #[schema(example = "image/jpeg")]
    pub output_mime_type: String,
}
