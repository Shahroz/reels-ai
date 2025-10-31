//! Defines the structure for a full tool response, including all properties.
//!
//! This struct is used to represent the complete output of a tool,
//! typically containing the tool's name and a flexible JSON value for its properties.
//! It is designed for internal processing where all tool output details are needed.
//! Adheres to one-item-per-file and FQN standards.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, schemars::JsonSchema)]
pub struct FullToolResponse {
    pub tool_name: std::string::String,
    pub response: serde_json::Value,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_full_tool_response_serde() {
        let json_properties = serde_json::json!({ "key": "value", "number": 123 });
        let response = super::FullToolResponse {
            tool_name: std::string::String::from("example_tool"),
            response: json_properties.clone(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: super::FullToolResponse = serde_json::from_str(&serialized).unwrap();

        std::assert_eq!(deserialized.tool_name, "example_tool");
        std::assert_eq!(deserialized.response, json_properties);
    }
}