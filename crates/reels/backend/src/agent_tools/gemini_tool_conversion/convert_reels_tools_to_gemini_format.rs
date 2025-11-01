//! Provides the main function to convert `ReelsToolParameters` into `GeminiTools`.
//!
//! This module iterates through all variants of `ReelsToolParameters`,
//! extracts their names, descriptions, and parameter schemas (by generating
//! a JSON schema via `schemars` and then converting it), and assembles them
//! into the `GeminiTools` structure expected by the Gemini API.
//! Adheres strictly to project Rust coding standards.

use anyhow::anyhow;
use llm::vendors::gemini::function_declaration::FunctionDeclaration;

/// Converts all defined `ReelsToolParameters` into a `GeminiTools` structure.
///
/// This function iterates over each variant of `ReelsToolParameters`:
/// 1. Derives the tool name (snake_case).
/// 2. Retrieves the tool description.
/// 3. Generates a JSON schema for the parameters of that specific tool variant.
/// 4. Converts this JSON schema into the `GeminiSchema` format.
/// 5. Assembles these into `GeminiFunctionDeclaration` objects.
///
/// # Returns
/// A `Result` containing `GeminiTools` if successful, or an error string if any part of the
/// conversion fails (e.g., schema generation or parsing). The `Tool` struct contains all declarations.
#[allow(clippy::too_many_lines)] // The match statement is inherently long.
pub fn convert_reels_tools_to_gemini_format() -> anyhow::Result<Vec<FunctionDeclaration>> {
    let mut declarations = Vec::new();
   // Requires ReelsToolParameters to derive strum::IntoEnumIterator, ToString, EnumProperties
   for tool_variant in <crate::agent_tools::reels_tool_parameters::ReelsToolParameters as strum::IntoEnumIterator>::iter() {
       let tool_name = tool_variant.to_string(); // Relies on #[strum(serialize_all = "snake_case")]
       let tool_description = strum::EnumProperty::get_str(&tool_variant, "description")
           .ok_or_else(|| {
               anyhow!("Missing description for tool variant: {:?}", tool_variant.as_ref())
           })?
            .to_string();

        // Generate JSON schema for the parameters of the specific variant.
        // This requires matching on the variant to know its specific parameter type.
        // Only reels generation related tools are supported.
        let params_json_schema_value = match tool_variant {
            // BrowseWithQuery: Used internally by generate_reel to fetch product information from URLs
            crate::agent_tools::reels_tool_parameters::ReelsToolParameters::BrowseWithQuery(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams)
                )
            }
            // GenerateReel: Core tool for generating reels (short videos)
            crate::agent_tools::reels_tool_parameters::ReelsToolParameters::GenerateReel(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::generate_reel_params::GenerateReelParams),
                )
            }
        }
        .map_err(|e| anyhow!("Failed to serialize schema to JSON for {}: {}", tool_name, e))?;

        declarations.push(
            llm::vendors::gemini::function_declaration::FunctionDeclaration {
                name: tool_name,
                description: tool_description,
                parameters: params_json_schema_value, // Use the serde_json::Value directly
            },
        );
    }
    // Wrap all function declarations into a single Tool object.
    // If no tools are defined, an empty Vec of declarations is fine.
    Ok(declarations)
}

