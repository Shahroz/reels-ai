//! Defines the parameters for the PropertyDescriptionToContents GN workflow tool.
//!
//! This struct holds the property description required for generating comprehensive
//! marketing content collection using the GenNodes workflow execution engine.
//! The PropertyDescriptionToContents workflow converts property descriptions into
//! various types of marketing content suitable for real estate purposes.
//! Used for strong typing in the PropertyDescriptionToContents tool handler and schema generation.

/// Parameters for the PropertyDescriptionToContents GN workflow tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    std::default::Default,
)]
pub struct PropertyDescriptionToContentsParams {
    /// Property description or information to convert into marketing content
    #[schema(example = "Beautiful 3-bedroom home with modern kitchen, hardwood floors, and spacious backyard. Located in quiet neighborhood with excellent schools nearby.")]
    pub property_info: std::string::String,

    /// Optional user ID for the request (injected by the system)
    #[schemars(skip)]
    pub user_id: std::option::Option<uuid::Uuid>,
} 