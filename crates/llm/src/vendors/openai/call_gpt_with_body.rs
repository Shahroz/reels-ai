//! Provides a function to call the OpenAI chat completion API with a pre-constructed request body.
//! 
//! This function takes a `serde_json::Value` representing the entire request body.
//! It retrieves the API key from the environment, sends the request using reqwest,
//! and returns the content of the first choice in the response as a String.
//! Uses anyhow for error handling and sets a default timeout.

pub async fn call_gpt_with_body(request_body: serde_json::Value) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(crate::constants::TIMEOUT))
        .build()
        .expect("Failed to build reqwest client with timeout"); // Consider returning Result
        
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY must be set in the environment"))?; // Use map_err for better error type
        
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", std::format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;

    // Check if the response status is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
        return std::result::Result::Err(anyhow::anyhow!("OpenAI API request failed with status {}: {}", status, error_body));
    }

    let response_json: serde_json::Value = response.json().await?;
    
    // More robust parsing of the response
    let output = response_json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .ok_or_else(|| anyhow::anyhow!("Failed to parse 'choices[0].message.content' from OpenAI response"))?
        .to_string();
        
    std::result::Result::Ok(output)
}

/// In-File Tests
#[cfg(test)]
mod tests {
    // Similar to call_gpt, tests require mocking.
    
    #[tokio::test] // Mark test as async
    async fn test_call_gpt_with_body_placeholder() {
        // Placeholder: Basic assertion
        // A real test would construct a Value body, set up a mock server,
        // call call_gpt_with_body, and assert the result.
        assert!(true); 
    }
}
