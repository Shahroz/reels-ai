//! Represents the result of a function call, to be sent back to the Gemini API.
//!
//! This structure is used within a `Part` when the client sends a message with
//! `role: "function"` to provide the output of a tool execution requested by the model.
//! It contains the name of the function that was called and its `response`, which
//! is itself a `Content` object.
//! Adheres to one-item-per-file and fully-qualified-path guidelines.

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct FunctionResultPart {
    /// The name of the function whose result is being provided.
    /// This should match the name from the `FunctionCallResponse` received from the model.
    pub name: std::string::String,

    /// The actual result of the function execution.
    /// This is structured as a `Content` object, which typically contains one or more `Part`s
    /// (e.g., a single `Part` with text representing the function's output, possibly as a JSON string).
    pub response: crate::vendors::gemini::content::Content,
}

#[cfg(test)]
mod tests {
    // Use super::FunctionResultPart for the struct under test.
    // Use std::string::String for String.
    // Use crate::vendors::gemini::content::Content and crate::vendors::gemini::part::Part.

    #[test]
    fn test_function_result_part_serialization() {
        let result_content = crate::vendors::gemini::content::Content {
            role: None,
            parts: std::vec![crate::vendors::gemini::part::Part {
                text: Some(std::string::String::from("{\"temperature\": 22, \"unit\": \"celsius\"}")),
                inline_data: None,
                file_data: None,
                function_response: None,
                function_call: None, // Assuming Part is updated by then, else remove
            }],
        };

        let func_result_part = super::FunctionResultPart {
            name: std::string::String::from("getCurrentWeather"),
            response: result_content,
        };

        let json_str = serde_json::to_string(&func_result_part).unwrap();

        // Expected structure based on field names.
        // The exact JSON string for `response` (Content struct) depends on its own serialization.
        let expected_json_fragment_name = r#""name":"getCurrentWeather""#;
        let expected_json_fragment_response_intro = r#""response":{"parts":[{"text":"{\"temperature\": 22, \"unit\": \"celsius\"}"#;


        assert!(json_str.contains(expected_json_fragment_name));
        assert!(json_str.contains(expected_json_fragment_response_intro));

        // For a more robust test, deserialize and check fields if Part is complex.
        // Here, we primarily check that `name` and `response` (as Content) are serialized.
        let deserialized: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.get("name").unwrap().as_str().unwrap(), "getCurrentWeather");
        assert!(deserialized.get("response").unwrap().is_object());
        assert!(deserialized.get("response").unwrap().get("parts").unwrap().is_array());
    }
}