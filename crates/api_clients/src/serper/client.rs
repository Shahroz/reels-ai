//! Provides a client function for interacting with the Serper API.
//!
//! This module contains the logic for making search requests to the Serper API.
//! It handles API key retrieval, request building, execution, and response handling.
//! Usage involves calling the `search` function with a query string.
//! Requires the `SERPER_API_KEY` environment variable to be set.

//! Revision History
//! - 2025-04-24T08:27:21Z @AI: Created file and extracted logic from main.rs.


/// Performs a search query using the Serper API.
///
/// # Arguments
///
/// * `query` - The search query string.
///
/// # Returns
///
/// * `Ok(String)` containing the response body text on success.
/// * `Err(Box<dyn std::error::Error>)` on failure (e.g., network error, API error, missing key).
pub async fn search(query: &str) -> std::result::Result<std::string::String, Box<dyn std::error::Error>> {
    ::dotenvy::dotenv().ok();
    // Read API key from environment
    let api_key = std::env::var("SERPER_API_KEY")?;
    // Build HTTP client and headers
    let client = reqwest::Client::builder().build()?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-KEY", api_key.parse()?);
    headers.insert("Content-Type", "application/json".parse()?);
    // Prepare JSON body
    let json_body = serde_json::json!({ "q": query });
    // Send request to Serper API
    let response = client
        .request(reqwest::Method::POST, "https://google.serper.dev/search")
        .headers(headers)
        .json(&json_body)
        .send()
        .await?;
    let body = response.text().await?;
    std::result::Result::Ok(body)
}

// Basic tests could be added here later, potentially mocking the HTTP request.
// For now, integration tests via the main binary are sufficient.
