//! Defines the schema for the parameters of a function declaration in the Gemini API.
//!
//! This specifies the overall structure (e.g., "OBJECT"), the individual properties
//! (parameters), and which of them are required.
//! Uses `PropertyDefinition` for its properties. Adheres to coding guidelines.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FunctionParametersSchema {
    #[serde(rename = "type")]
    pub schema_type: String, // Field name `schema_type` in Rust, JSON key is "type"
    pub properties: std::collections::HashMap<String, crate::vendors::gemini::property_definition::PropertyDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    //! Tests for FunctionParametersSchema serialization and deserialization.

    #[test]
    fn test_function_parameters_schema_serialization() {
        //! Verifies that FunctionParametersSchema serializes correctly to JSON.
        let mut properties = std::collections::HashMap::new();
        properties.insert(
            std::string::String::from("location"),
            crate::vendors::gemini::property_definition::PropertyDefinition {
                r#type: std::string::String::from("string"),
                description: Some(std::string::String::from("City and state")),
            },
        );
        properties.insert(
            std::string::String::from("unit"),
            crate::vendors::gemini::property_definition::PropertyDefinition {
                r#type: std::string::String::from("string"),
                description: None, // Optional description
            },
        );

        let schema = super::FunctionParametersSchema {
            schema_type: std::string::String::from("OBJECT"),
            properties,
            required: Some(std::vec![std::string::String::from("location")]),
        };

        // Using serde_json::to_value for robust comparison of HashMaps
        let json_value = serde_json::to_value(&schema).unwrap();
        let expected_json = serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "location": { "type": "string", "description": "City and state" },
                "unit": { "type": "string" } // "description" is omitted as it's None and skip_serializing_if
            },
            "required": ["location"]
        });
        assert_eq!(json_value, expected_json);
    }

    #[test]
    fn test_function_parameters_schema_deserialization() {
        //! Verifies that FunctionParametersSchema deserializes correctly from JSON.
        let json_data = r#"{
            "type": "OBJECT",
            "properties": {
                "query": { "type": "string", "description": "The search query" },
                "limit": { "type": "integer" }
            },
            "required": ["query"]
        }"#;
        let schema: super::FunctionParametersSchema = serde_json::from_str(json_data).unwrap();

        assert_eq!(schema.schema_type, "OBJECT");
        assert_eq!(schema.properties.len(), 2);
        
        let query_prop = schema.properties.get("query").unwrap();
        assert_eq!(query_prop.r#type, "string");
        assert_eq!(query_prop.description, Some(std::string::String::from("The search query")));
        
        let limit_prop = schema.properties.get("limit").unwrap();
        assert_eq!(limit_prop.r#type, "integer");
        assert!(limit_prop.description.is_none());
        
        assert_eq!(schema.required, Some(std::vec![std::string::String::from("query")]));
    }

    #[test]
    fn test_function_parameters_schema_optional_required_deserialization() {
        //! Verifies deserialization when "required" is missing in JSON.
        let json_data = r#"{
            "type": "OBJECT",
            "properties": {
                "optional_param": { "type": "boolean" }
            }
        }"#;
        let schema: super::FunctionParametersSchema = serde_json::from_str(json_data).unwrap();
        assert_eq!(schema.schema_type, "OBJECT");
        assert!(schema.required.is_none());
    }
}