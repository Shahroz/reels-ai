//! Defines parameters for the `generate_reel` agent tool.
//!
//! This structure encapsulates the request payload for generating a reel (short video)
//! from a product/service URL or text description, with a specified time duration.
//! The tool can fetch product information from a URL if provided, or use the description
//! directly to create engaging reel content.

#[derive(
    std::fmt::Debug,
    std::clone::Clone,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    utoipa::ToSchema,
    std::default::Default,
)]
pub struct GenerateReelParams {
    /// The prompt or description for the reel content. This can be a product description,
    /// service overview, or any text that describes what the reel should showcase.
    /// If a URL is provided, this prompt will be used to guide the content generation
    /// after fetching the URL content.
    #[schema(example = "Create an engaging reel showcasing a modern smartphone with advanced camera features")]
    pub prompt: std::string::String,

    /// Optional URL to a product or service page. If provided, the tool will fetch
    /// and analyze the page content to gather product/service information.
    #[schema(example = "https://example.com/product/smartphone")]
    pub product_url: std::option::Option<std::string::String>,

    /// Time duration for the reel in seconds. Common values: 10, 15, 20, 30, 60.
    /// The reel will be generated to match this exact duration.
    #[schema(example = 30, minimum = 5, maximum = 120)]
    pub time_range_seconds: i32,

    /// Optional user ID (injected by the system)
    #[schemars(skip)]
    pub user_id: std::option::Option<uuid::Uuid>,

    /// Optional organization ID to deduct credits from organization instead of user
    #[schemars(skip)]
    pub organization_id: std::option::Option<uuid::Uuid>,
}

