//! Defines the parameters for the Narrativ 'browse_raw' tool.
//!
//! This struct holds the URL to be browsed for its raw content. It's used
//! for strong typing in Narrativ's browse_raw tool handler and for schema generation.

/// Parameters for the 'browse_raw' tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    std::default::Default,
    schemars::JsonSchema,
    utoipa::ToSchema
)]
pub struct BrowseRawParams {
    /// The URL of the website to browse.
    #[schema(example = "https://example.com")]
    pub url: std::string::String,
}