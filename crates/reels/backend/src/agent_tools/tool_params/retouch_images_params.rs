//! Defines the parameters for the RetouchImages GN workflow tool.
//!
//! This struct holds the photo URIs and optional retouch prompt required 
//! for retouching images using the GenNodes workflow execution engine.
//! The RetouchImages workflow retouches a list of images from GCS using a prompt 
//! and DALL-E processing capabilities.
//! Used for strong typing in the RetouchImages tool handler and schema generation.

/// Parameters for the RetouchImages GN workflow tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    std::default::Default,
)]
pub struct RetouchImagesParams {
    /// A list of GCS URIs for the images to be retouched (e.g., gs://bucket/photo.jpg)
    #[schema(example = json!(["gs://my-bucket/images/photo1.jpg", "gs://my-bucket/images/photo2.jpg"]))]
    pub photos: std::vec::Vec<std::string::String>,

    /// An optional prompt to guide the DALL-E retouching process
    #[schema(example = "Enhance the lighting and remove blemishes")]
    pub retouch_prompt: std::option::Option<std::string::String>,

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