//! Defines the definition for a single property (parameter) in a JSON schema for function parameters.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PropertyDefinition {
    /// The type of the property (e.g., "string", "integer").
    #[serde(rename = "type")]
    pub r#type: String,
    /// An optional description of the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}