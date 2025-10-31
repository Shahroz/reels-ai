//! Provides the function to interact with the Replicate API for model predictions.
//!
//! This file contains the `call_replicate_api` asynchronous function.
//! It handles initiating a prediction request and polling for the results.
//! Includes necessary internal helper structs for deserialization.
//! Conforms to the one-item-per-file standard and uses fully qualified paths.

//! Revision History
//! - 2025-04-15T15:27:38Z @AI: Initial creation during refactor.

use crate::constants::TIMEOUT;

// Helper structs for deserialization - kept private within this file.
#[derive(serde::Deserialize)]
struct PredictionResponse {
    urls: PredictionUrls,
}

#[derive(serde::Deserialize)]
struct PredictionUrls {
    get: std::string::String,
}

#[derive(serde::Deserialize)]
struct PollResponse {
    status: std::string::String,
    output: Option<std::vec::Vec<std::string::String>>,
    error: Option<std::string::String>,
}

/// Calls the Replicate API with the specified prompt and model, and polls for the prediction result.
///
/// This function reads the API token from the REPLICATE_API_KEY environment variable, sends a POST request to initiate
/// a prediction, and continuously polls the provided GET URL until the prediction status is either "succeeded" or "failed".
/// It joins the output array into a single string if the prediction is successful.
pub async fn call_replicate_api(prompt: &str, model: &crate::vendors::replicate::replicate_model::ReplicateModel) -> anyhow::Result<std::string::String> {
    let token = std::env::var("REPLICATE_API_KEY").map_err(|_| anyhow::anyhow!("REPLICATE_API_KEY not set"))?;
    let client = reqwest::Client::new();
    let endpoint = model.endpoint();

    // Prepare payload with stream disabled for simplicity.
    let payload = serde_json::json!({
        "stream": false,
        "input": {
            "prompt": prompt
        }
    });

    // Initiate prediction via POST.
    let resp = client.post(endpoint)
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let pred: PredictionResponse = resp.json().await?;
    let poll_url = pred.urls.get;

    let timeout = std::time::Duration::from_secs(TIMEOUT);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            return std::result::Result::Err(anyhow::anyhow!("Prediction timed out"));
        }
        let poll_resp = client.get(&poll_url)
            .bearer_auth(&token)
            .send()
            .await?
            .error_for_status()?;

        let poll: PollResponse = poll_resp.json().await?;

        if poll.status == "succeeded" {
            if let Some(output) = poll.output {
                 return std::result::Result::Ok(std::string::String::from_iter(output));
            } else {
                return std::result::Result::Err(anyhow::anyhow!("No output in prediction"));
            }
        } else if poll.status == "failed" {
             return std::result::Result::Err(anyhow::anyhow!("Prediction failed: {}", poll.error.unwrap_or_else(|| "Unknown error".to_string())));
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

// In-File Tests
#[cfg(test)]
mod tests {
    // Tests would require mocking the environment variable and HTTP requests.
    // Placeholder for future implementation.

    // Example test structure (would need mocking library like `mockito` or `wiremock`)
    // #[tokio::test]
    // async fn test_call_replicate_api_success() {
    //     // Setup mock server for POST and GET endpoints
    //     // Set REPLICATE_API_KEY env var for test scope
    //     // Define mock responses
    //
    //     // let model = crate::vendors::replicate::replicate_model::ReplicateModel::Mistral7bV01;
    //     // let prompt = "test prompt";
    //     // let result = super::call_replicate_api(prompt, &model).await;
    //     // assert!(result.is_ok());
    //     // assert_eq!(result.unwrap(), "mocked successful output");
    //
    //     // Clean up env var if necessary
    // }

    // #[tokio::test]
    // async fn test_call_replicate_api_failure() {
    //     // Setup mock server for POST and GET endpoints (returning failed status)
    //     // ...
    //     // let result = super::call_replicate_api(prompt, &model).await;
    //     // assert!(result.is_err());
    //     // assert!(result.unwrap_err().to_string().contains("Prediction failed"));
    // }

    // #[tokio::test]
    // async fn test_call_replicate_api_timeout() {
    //     // Setup mock server that delays response or never returns success/fail
    //     // ...
    //     // IMPORTANT: Need to adjust timeout in the function or test setup for practicality
    //     // let result = super::call_replicate_api(prompt, &model).await;
    //     // assert!(result.is_err());
    //     // assert!(result.unwrap_err().to_string().contains("Prediction timed out"));
    // }
}
