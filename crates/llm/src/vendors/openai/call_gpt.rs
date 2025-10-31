//! Provides a function to call the OpenAI chat completion API with basic parameters.
//! 
//! This function takes a model name, API key, and a list of messages (as JSON Values).
//! It constructs the request, sends it to the OpenAI API endpoint using reqwest,
//! and returns the content of the first choice in the response as a String.
//! Uses anyhow for error handling and sets a default timeout.

pub async fn call_gpt(model: &str, api_key: &str, messages: std::vec::Vec<serde_json::Value>) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(crate::constants::TIMEOUT))
        .build()
        .expect("Failed to build reqwest client with timeout"); // Consider returning Result instead of expect
    
    let request_body = serde_json::json!({
        "model": model,
        "messages": messages
    });

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
    // Tests would ideally use mocking (e.g., with mockito or wiremock) 
    // to simulate the OpenAI API responses without actual network calls.
    
    #[tokio::test] // Mark test as async
    async fn test_call_gpt_placeholder() {
        // Placeholder: Basic assertion
        // A real test would set up a mock server, call call_gpt, 
        // and assert the expected result based on the mock response.
        assert!(true); 
    }
}
