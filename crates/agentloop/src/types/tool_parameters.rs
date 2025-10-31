//! Defines the possible parameter structures for different tools.
//!
//! This enum encapsulates the specific arguments required by each tool,
//! replacing the generic `serde_json::Value` previously used in `ToolChoice`.
//! Conforms to one-item-per-file and FQN standards.
//! Ensures type safety and schema generation for tool parameters.

use utoipa::ToSchema;

/// Enum representing the parameters for various tools.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, ToSchema)]
// Use serde tag/content representation for clean JSON serialization.
#[serde(tag = "tool_name", content = "parameters")]
pub enum ToolParameters {
    /// Parameters for the 'search' tool.
    Search { query: std::string::String},
    SaveContext {
        content: std::string::String,
        source: Option<std::string::String>,
    },
    /// Parameters for the 'browse' tool (can handle file URLs too).
    Browse { url: std::string::String },
    // Add other tool parameter variants here as needed.
    // Example: FileWrite { path: String, content: String },
}

#[cfg(test)]
mod tests {
    // Test serialization and deserialization for each variant.
    #[test]
    fn test_search_params_serde() {
        let params = super::ToolParameters::Search {
            query: std::string::String::from("latest AI news"),
        };
        let serialized = serde_json::to_string(&params).unwrap();
        // Expected JSON: {"tool_name":"Search","parameters":{"query":"latest AI news"}}
        let deserialized: super::ToolParameters = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            super::ToolParameters::Search { query } => {
                std::assert_eq!(query, "latest AI news");
            }
            _ => std::panic!("Deserialized into wrong variant"),
        }
    }

    #[test]
    fn test_browse_params_serde() {
        let params = super::ToolParameters::Browse {
            url: std::string::String::from("file://rust_guidelines"),
        };
        let serialized = serde_json::to_string(&params).unwrap();
        // Expected JSON: {"tool_name":"Browse","parameters":{"url":"file://rust_guidelines"}}
        let deserialized: super::ToolParameters = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            super::ToolParameters::Browse { url } => {
                std::assert_eq!(url, "file://rust_guidelines");
            }
            _ => std::panic!("Deserialized into wrong variant"),
        }
    }
}