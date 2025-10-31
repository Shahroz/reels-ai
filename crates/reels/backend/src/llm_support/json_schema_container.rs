//! Defines JsonSchemaContainer, a wrapper for `serde_json::Value` for typed LLM interaction.
//!
//! This struct is used with `llm_typed` to facilitate the generation or modification
//! of JSON schemas by an LLM. It implements `FewShotsOutput` to provide examples
//! of valid JSON schemas, including custom properties like `x-column-ordering` and `x-column-groups`.
//! Adheres to `rust_guidelines.md` (one item per file, FQNs, preamble).

/// A structured representation of a JSON schema for typed LLM interaction.
///
/// This struct explicitly defines common JSON schema fields like `type`, `properties`, and `required`,
/// along with custom extensions such as `x-column-ordering` and `x-column-groups`.
/// It is designed for use with `llm_typed` to enable an LLM to generate or modify
/// JSON schemas in a strongly-typed manner.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct JsonSchemaContainer {
    /// The JSON schema type (e.g., "object", "array", "string").
    #[serde(rename = "type")]
    pub schema_type: String,

    /// A map of property names to their schema definitions (which are `serde_json::Value`).
    pub properties: std::collections::HashMap<String, serde_json::Value>,

    /// An optional list of required property names.
    pub required: Option<std::vec::Vec<String>>,

    /// An optional list specifying the preferred order of columns/properties in a UI.
    #[serde(rename = "x-column-ordering")]
    pub x_column_ordering: Option<std::vec::Vec<String>>,

    /// An optional map defining groups of related columns/properties for UI rendering.
    /// The key is the group name, and the value is a list of column/property names in that group.
    #[serde(rename = "x-column-groups")]
    pub x_column_groups: Option<std::collections::HashMap<String, std::vec::Vec<String>>>,
}

impl llm::few_shots_traits::FewShotsOutput<JsonSchemaContainer> for JsonSchemaContainer {
    fn few_shots() -> std::vec::Vec<JsonSchemaContainer> {
        std::vec![
            JsonSchemaContainer {
                schema_type: "object".to_string(),
                properties: serde_json::from_value(serde_json::json!({
                        "name": {"type": "string", "description": "Name of the item"},
                        "age": {"type": "integer", "minimum": 0},
                        "email": {"type": "string", "format": "email"}
                })).unwrap(),
                required: Some(vec!["name".to_string(), "age".to_string()]),
                x_column_ordering: Some(vec![
                    "name".to_string(),
                    "age".to_string(),
                    "email".to_string()
                ]),
                x_column_groups: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "User Information".to_string(),
                        vec!["name".to_string(), "email".to_string()],
                    );
                    map.insert("Details".to_string(), vec!["age".to_string()]);
                    map
                }),
            },
            JsonSchemaContainer {
                schema_type: "object".to_string(),
                properties: serde_json::from_value(serde_json::json!({
                        "product_id": {"type": "string", "description": "Unique identifier for the product"},
                        "description": {"type": "string"},
                        "price": {"type": "number", "format": "float"},
                        "in_stock": {"type": "boolean", "default": true}
                })).unwrap(),
                required: Some(vec!["product_id".to_string(), "price".to_string()]),
                x_column_ordering: Some(vec![
                    "product_id".to_string(),
                    "description".to_string(),
                    "price".to_string(),
                    "in_stock".to_string()
                ]),
                x_column_groups: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "Identification".to_string(),
                        vec!["product_id".to_string(), "description".to_string()],
                    );
                    map.insert(
                        "Financials".to_string(),
                        vec!["price".to_string(), "in_stock".to_string()],
                    );
                    map
                }),
            },
            JsonSchemaContainer {
                schema_type: "object".to_string(),
                properties: serde_json::from_value(serde_json::json!({
                        "metadata_version": {"type": "string", "enum": ["1.0", "1.1", "2.0"]},
                        "tags": {"type": "array", "items": {"type": "string"}},
                        "config": {
                            "type": "object",
                            "properties": {
                                "retries": {"type": "integer", "default": 3}
                            }
                        }
                })).unwrap(),
                required: None,
                x_column_ordering: Some(vec![
                    "metadata_version".to_string(),
                    "tags".to_string(),
                    "config".to_string()
                ]),
                x_column_groups: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        "Core Metadata".to_string(),
                        vec!["metadata_version".to_string(), "tags".to_string()],
                    );
                    map.insert("Runtime Config".to_string(), vec!["config".to_string()]);
                    map
                }),
            },
        ]
    }
}
