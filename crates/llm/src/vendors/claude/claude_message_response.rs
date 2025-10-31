//! Defines the response structure from the Claude Messages API.
//!
//! This struct represents the JSON response received from a `/v1/messages` call.
//! Includes metadata like ID, type, role, model, and the assistant's content.
//! Also contains stop reason/sequence and token usage information.
//! Uses other defined types like `ContentBlock`, `Usage`.

/// Represents the response body from the Claude Messages API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClaudeMessageResponse {
    pub id: std::string::String,
    #[serde(rename = "type")]
    pub response_type: std::string::String, // e.g., "message"
    pub role: std::string::String, // Should be "assistant"
    pub model: std::string::String, // The model that handled the request
    pub content: std::vec::Vec<crate::vendors::claude::content_block::ContentBlock>, // Assistant's response content
    pub stop_reason: std::string::String, // e.g., "end_turn", "max_tokens"
    pub stop_sequence: Option<std::string::String>,
    pub usage: crate::vendors::claude::usage::Usage,
}

#[cfg(test)]
mod tests {
    // Use super::* for the item under test (ClaudeMessageResponse).
    // Use fully qualified paths for everything else.

    #[test]
    fn test_response_deserialization() {
        // Load environment variables if needed for context, though this test is self-contained.
        // dotenvy::dotenv().ok(); // Not strictly needed here if test data is static JSON

        let response_json = r#"{
            "id": "msg_01JHxbjBNcZG9hMDo4sqKHhn",
            "type": "message",
            "role": "assistant",
            "model": "claude-3-opus-20240229",
            "content": [
                {
                    "type": "text",
                    "text": "Hello! How can I assist you today?"
                }
            ],
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 12
            }
        }"#;
        let response: super::ClaudeMessageResponse = serde_json::from_str(response_json).unwrap();

        // Print for debugging if needed
        // std::println!("{:?}", response);

        assert_eq!(response.id, "msg_01JHxbjBNcZG9hMDo4sqKHhn");
        assert_eq!(response.role, "assistant");
        assert_eq!(response.model, "claude-3-opus-20240229");
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 12);
        assert!(matches!(response.content.first().unwrap(), crate::vendors::claude::content_block::ContentBlock::Text { text } if text == "Hello! How can I assist you today?"));
    }
}
