//! Defines the parameters for the VocalTour GN workflow tool.
//!
//! This struct holds the optional file URIs required for researching a property 
//! using the GenNodes workflow execution engine. The VocalTour workflow analyzes 
//! property information using AI analysis of provided files (videos, photos, documents) 
//! to gather information and populate a Property Description.
//! Used for strong typing in the VocalTour tool handler and schema generation.

/// Parameters for the VocalTour GN workflow tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    std::default::Default,
)]
pub struct VocalTourParams {
    /// Optional URIs of document files for analysis (e.g., gs://bucket/document.pdf)
    #[schema(example = json!(["gs://my-bucket/property-docs/deed.pdf", "gs://my-bucket/property-docs/contract.pdf"]))]
    pub documents: std::option::Option<std::vec::Vec<std::string::String>>,

    /// Optional URIs of image files for analysis (e.g., gs://bucket/photo.jpg)
    #[schema(example = json!(["gs://my-bucket/property-photos/exterior.jpg", "gs://my-bucket/property-photos/interior.jpg"]))]
    pub photos: std::option::Option<std::vec::Vec<std::string::String>>,

    /// Optional prompt to retouch the extracted frames using DALL-E
    #[schema(example = "Make the property look more luxurious and well-maintained")]
    pub retouch_prompt: std::option::Option<std::string::String>,

    /// Optional URIs of video files for analysis (e.g., gs://bucket/video.mp4)
    #[schema(example = json!(["gs://my-bucket/property-videos/walkthrough.mp4", "gs://my-bucket/property-videos/overview.mp4"]))]
    pub videos: std::option::Option<std::vec::Vec<std::string::String>>,

    /// Optional user ID for the request (injected by the system)
    #[schemars(skip)]
    pub user_id: std::option::Option<uuid::Uuid>,

    /// Optional organization ID to deduct credits from organization instead of user
    #[schemars(skip)]
    pub organization_id: std::option::Option<uuid::Uuid>,
} 