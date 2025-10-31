//! Represents a function call suggested by the Gemini API.
//!
//! This structure is part of a `PartResponse` when the model decides to call a function.
//! It includes the name of the function to be called and its arguments.
//! Adheres to one-item-per-file and fully-qualified-path guidelines.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallResponse {
    pub name: std::string::String,
    // Using serde_json::Value for args to handle arbitrary JSON object.
    pub args: serde_json::Value,
}

#[cfg(test)]
mod tests {
    // Use super::FunctionCallResponse for the struct under test.
    // Use std::string::String for String.
    // Use serde_json for JSON manipulation.

    #[test]
    fn test_function_call_response_serialization() {
        let call = super::FunctionCallResponse {
            name: std::string::String::from("test_function"),
            args: serde_json::json!({ "param1": "value1", "param2": 123 }),
        };
        let json_str = serde_json::to_string(&call).unwrap();
        // Basic check for serialization
        assert!(json_str.contains("test_function"));
        assert!(json_str.contains("param1"));

        // A more robust test deserializes and checks fields.
        let deserialized_call: super::FunctionCallResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized_call.name, "test_function");
    }

    #[test]
    fn test_function_call_response_deserialization() {
        let json_data = r#"{
            "name": "another_func",
            "args": {
                "arg_bool": true,
                "arg_array": [1, "two"]
            }
        }"#;
        let call: super::FunctionCallResponse = serde_json::from_str(json_data).unwrap();
        assert_eq!(call.name, "another_func");
        assert_eq!(call.args["arg_bool"], serde_json::json!(true));
        assert_eq!(call.args["arg_array"], serde_json::json!([1, "two"]));
    }
}