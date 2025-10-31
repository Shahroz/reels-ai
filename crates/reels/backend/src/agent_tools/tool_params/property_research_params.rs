//! Defines the parameters for the PropertyResearch GN workflow tool.
//!
//! This struct holds the property identifier and optional file URIs required 
//! for researching a property using the GenNodes workflow execution engine.
//! The PropertyResearch workflow analyzes property information using web search,
//! AI analysis of provided files, and Perplexity to populate an MLS Entry form.
//! Used for strong typing in the PropertyResearch tool handler and schema generation.

/// Parameters for the PropertyResearch GN workflow tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    std::default::Default,
)]
pub struct PropertyResearchParams {
    /// The name or address of the property to research (e.g., "123 Main St, City, State" or "123 Main Street")
    #[schema(example = "123 Main Street, Seattle, WA")]
    pub property_identifier: std::string::String,

    /// Optional URIs of document files for analysis (e.g., gs://bucket/document.pdf)
    #[schema(example = json!(["gs://my-bucket/property-docs/deed.pdf", "gs://my-bucket/property-docs/contract.pdf"]))]
    pub documents: std::option::Option<std::vec::Vec<std::string::String>>,

    /// Optional URIs of image files for analysis (e.g., gs://bucket/photo.jpg)
    #[schema(example = json!(["gs://my-bucket/property-photos/exterior.jpg", "gs://my-bucket/property-photos/interior.jpg"]))]
    pub photos: std::option::Option<std::vec::Vec<std::string::String>>,

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