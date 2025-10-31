//! Defines the parameters for the Quick Enhance Image tool.
//!
//! This struct holds the image data and enhancement prompt required 
//! for enhancing images using various image enhancement models.
//! Currently uses the Gemini 2.5 Flash Image model (Nano Banana) but is designed
//! to be flexible for future model integrations.
//! The tool processes images directly without creating asset records,
//! returning the enhanced image data in the response.
//! Used for strong typing in the Quick Enhance Image tool handler and schema generation.

/// Parameters for the Quick Enhance Image tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    std::default::Default,
)]
pub struct QuickEnhanceImageParams {
    /// Base64 encoded image data to be enhanced (alternative to asset_id)
    #[schema(example = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD...")]
    pub image_data: std::option::Option<std::string::String>,

    /// Asset ID to fetch image from GCS (alternative to image_data)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub asset_id: std::option::Option<uuid::Uuid>,

    /// Enhancement prompt describing the desired modifications
    #[schema(example = "Enhance the lighting, remove blemishes, and improve overall quality")]
    pub enhancement_prompt: std::string::String,

    /// Optional MIME type for the enhanced image output (e.g., "image/jpeg", "image/png")
    #[schema(example = "image/jpeg")]
    pub output_mime_type: std::option::Option<std::string::String>,

    /// Optional user ID for the request (injected by the system)
    #[schemars(skip)]
    pub user_id: std::option::Option<uuid::Uuid>,

    /// Optional organization ID to deduct credits from organization instead of user
    #[schemars(skip)]
    pub organization_id: std::option::Option<uuid::Uuid>,

    /// Optional credit deduction parameters (if not provided, will be constructed from defaults)
    #[schemars(skip)]
    pub credit_changes_params: std::option::Option<crate::queries::user_credit_allocation::CreditChangesParams>,
}
