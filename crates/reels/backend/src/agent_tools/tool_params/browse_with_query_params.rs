//! Defines the parameters for the Reels 'browse_with_query' tool.
//!
//! This struct holds the URL to be browsed and a query to process the
//! content. It's used for strong typing in the tool handler and for schema generation.

/// Parameters for the 'browse_with_query' tool.
#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    std::default::Default,
    schemars::JsonSchema,
    utoipa::ToSchema
)]
pub struct BrowseWithQueryParams {
    /// The URL of the website to browse.
    #[schema(example = "https://example.com")]
    pub url: std::string::String,
    /// The query to process the website content with, to summarize or extract key information.
    #[schema(example = "What are the main products of this company?")]
    pub query: std::string::String,
}