//! Defines an enum to represent Reels generation tool parameters for direct deserialization.
//!
//! This enum uses `#[serde(untagged)]` to allow `serde_json::from_value` to attempt
//! deserialization of `tool_choice.parameters` into one of its variants based on the
//! distinct structure of the parameter types.
//! Adheres strictly to project Rust coding standards.
//!
//! Only tools related to reel generation are included.

use schemars::JsonSchema;
use strum_macros::{AsRefStr, Display, EnumIter, EnumProperty};

/// Enum representing the parameters for Reels generation tools.
///
/// Used for deserializing the `parameters` field from an `agentloop::types::tool_choice::ToolChoice`.
/// The `#[serde(untagged)]` attribute means Serde will try to deserialize into each variant
/// in order until one succeeds. This requires the parameter structs to be structurally distinct.
#[derive(
    serde::Deserialize, std::fmt::Debug, JsonSchema, EnumIter, Display, EnumProperty, AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum ReelsToolParameters {
   #[schemars(description = "Fetches and processes web page content based on a query. Used internally by generate_reel to fetch product information from URLs.")]
   #[strum(props(description = "Fetches and processes web page content based on a query. Used internally by generate_reel to fetch product information from URLs."))]
   BrowseWithQuery(crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams),
   #[schemars(description = "Generates a reel (short video) from a product/service URL or text description with a specified time duration. Fetches product information from URL if provided, then creates an engaging video montage using the video-to-montage service. Returns the GCS URL of the generated reel.")]
   #[strum(props(description = "Generates a reel (short video) from a product/service URL or text description with a specified time duration. Fetches product information from URL if provided, then creates an engaging video montage using the video-to-montage service. Returns the GCS URL of the generated reel."))]
   GenerateReel(crate::agent_tools::tool_params::generate_reel_params::GenerateReelParams),
}

#[cfg(test)]
mod tests {
    // Use super::* to access the item in the parent module (the file scope)
    use schemars::schema_for;
    use serde_json::json;
    use strum::EnumProperty;

    #[test]
    fn test_schema() {
        let schema = schema_for!(super::ReelsToolParameters);
        // Basic assertion to ensure schema generation runs
        assert!(serde_json::to_string_pretty(&schema).is_ok());
    }

    #[test]
    fn test_parsing() {
        let json = json!({
            "generate_reel": {
                "prompt": "Create an engaging reel showcasing a modern smartphone",
                "time_range_seconds": 30,
            }
        });

        let reels_tool_params_result: std::result::Result<
            crate::agent_tools::reels_tool_parameters::ReelsToolParameters,
            serde_json::Error,
        > = serde_json::from_value(json.clone());

        println!("{:?}", reels_tool_params_result);
    }
}

