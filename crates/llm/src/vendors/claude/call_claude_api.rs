//! Provides the function to call the Anthropic Claude Messages API.
//!
//! This asynchronous function handles the HTTP request to the Claude API endpoint.
//! It requires the API key from the environment and a serialized request body.
//! Returns the assistant's response text or an error.
//! Manages headers, client creation, sending, and response parsing.

use anyhow::Context; // Keep specific use for Context trait method if preferred over FQN

/// Calls the Claude Messages API.
///
/// # Arguments
///
/// * `request_body` - A `serde_json::Value` representing the serialized `ClaudeMessageRequest`.
///
/// # Returns
///
/// A `Result` containing the assistant's response text as a `String`, or an error.
pub async fn call_claude_api(request_body: serde_json::Value) -> anyhow::Result<std::string::String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    let client = reqwest::Client::new();
    // Use constants defined in separate files via the parent module's re-exports
    let url = format!("{}/v1/messages", crate::vendors::claude::CLAUDE_API_BASE_URL);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-api-key", reqwest::header::HeaderValue::from_str(&api_key)?);
    headers.insert("anthropic-version", reqwest::header::HeaderValue::from_static(crate::vendors::claude::CLAUDE_API_VERSION));
    headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));

    let response = client
        .post(&url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        return Err(anyhow::anyhow!(
            "Claude API request failed with status {}: {}",
            status,
            error_body
        ));
    }

    let response_body = response
        .json::<crate::vendors::claude::claude_message_response::ClaudeMessageResponse>()
        .await
        .context("Failed to parse Claude API response")?;

    // Extract the text content from the first text block, assuming it exists.
    // A more robust implementation might handle multiple blocks or different types.
    response_body
        .content
        .iter()
        .filter_map(|block| match block {
            crate::vendors::claude::content_block::ContentBlock::Text { text } => Some(text.clone()),
            // _ => None, // Ignore other block types for now
        })
        .next()
        .context("No text content found in Claude API response")
}


#[cfg(test)]
mod tests {
    // Use super::* for the item under test (call_claude_api).
    // Use fully qualified paths for everything else.

    // To run this test:
    // 1. Create a .env file in the root of the project (or relevant parent)
    // 2. Add ANTHROPIC_API_KEY=your_actual_api_key to the .env file
    // 3. Run `cargo test --package llm --lib vendors::claude::call_claude_api::tests::test_call_claude_api -- --nocapture`
    #[tokio::test]
    #[ignore] // Ignored by default to avoid running API calls unless explicitly requested
    async fn test_call_claude_api() {
        dotenvy::dotenv().ok(); // Load .env file

        // Ensure API key is available before running the test
        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            println!("Skipping test_call_claude_api: ANTHROPIC_API_KEY not set.");
            return;
        }

        let request = crate::vendors::claude::claude_message_request::ClaudeMessageRequest {
             // Use fully qualified path for ClaudeModel
            model: crate::vendors::claude::claude_model::ClaudeModel::Claude37SonnetLatest,
            messages: vec![crate::vendors::claude::message::Message {
                role: "user".to_string(),
                content: vec![crate::vendors::claude::content_block::ContentBlock::Text {
                    text: "Hello, Claude! Write a short haiku about Rust.".to_string(),
                }],
            }],
            max_tokens: 50,
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
        };

        let request_value = serde_json::to_value(request).expect("Failed to serialize request");

        match super::call_claude_api(request_value).await {
            Ok(response_text) => {
                println!("Claude API Response Text:\n{}", response_text);
                assert!(!response_text.is_empty());
            }
            Err(e) => {
                // Use eprintln for errors
                eprintln!("Error calling Claude API: {:?}", e);
                panic!("API call failed: {:?}", e);
            }
        }
    }
}
