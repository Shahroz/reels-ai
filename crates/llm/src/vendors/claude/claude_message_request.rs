//! Defines the request structure for the Claude Messages API.
//!
//! This struct encapsulates all parameters for a `/v1/messages` API call.
//! Includes the model, message history, max tokens, and optional parameters.
//! Supports serialization into the JSON format expected by the API.
//! Uses other defined types like `ClaudeModel`, `Message`.

/// Represents the request body for the Claude Messages API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClaudeMessageRequest {
    pub model: crate::vendors::claude::claude_model::ClaudeModel,
    pub messages: std::vec::Vec<crate::vendors::claude::message::Message>,
    pub max_tokens: u32,
    // Add other optional parameters like system prompt, temperature, etc. as needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<std::string::String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<std::vec::Vec<std::string::String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    // Add tool usage fields if needed
}


#[cfg(test)]
mod tests {
    // Use super::* for the item under test (ClaudeMessageRequest).
    // Use fully qualified paths for everything else.

    #[test]
    fn test_request_serialization() {
        let request = super::ClaudeMessageRequest {
            model: crate::vendors::claude::claude_model::ClaudeModel::Claude3Haiku,
            messages: vec![crate::vendors::claude::message::Message {
                role: "user".to_string(),
                content: vec![crate::vendors::claude::content_block::ContentBlock::Text {
                    text: "Hi".to_string(),
                }],
            }],
            max_tokens: 10,
            system: Some("Be brief".to_string()),
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
        };
        let json_value = serde_json::to_value(request).unwrap();
        let expected = serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Hi"
                        }
                ]
                }
            ],
           "max_tokens": 10,
           "system": "Be brief",
           "temperature": 0.7f32
       });
       assert_eq!(json_value, expected);
   }
}
