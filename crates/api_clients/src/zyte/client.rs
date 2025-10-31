//! Main client for interacting with the Zyte API.
//!
//! Implements the ZyteClient struct, including its construction (`new`)
//! and the primary method (`extract`) for making API requests.
//! Uses fully qualified paths and includes in-file tests.

//! Revision History
//! - 2025-04-15T15:06:21Z @AI: Created file based on one-item-per-file guideline, moved impl and tests.
//! - 2025-04-15T15:01:49Z @AI: Implement new() and extract() (original history from client.rs).

use base64::engine::{general_purpose::STANDARD, Engine as _};

/// Represents the client for making requests to the Zyte API.
#[derive(Clone)]
#[derive(Debug)]
pub struct ZyteClient {
    api_key: String,
    client: ::reqwest::Client,
}

impl ZyteClient {
    //! Creates a new ZyteClient by loading the API key and building the HTTP client.
    //! 
    //! Loads environment variables, retrieves ZYTE_API_KEY, and builds a reqwest::Client.
    //! Returns ZyteError on failure (e.g., key not found, client build error).
    pub fn new() -> Result<Self, crate::zyte::error::ZyteError> {
        // Load environment variables
        ::dotenvy::dotenv().ok();
        
        // Retrieve the API key
        let api_key = ::std::env::var("ZYTE_API_KEY")
            .map_err(|_| crate::zyte::error::ZyteError::EnvVarError(String::from("ZYTE_API_KEY not found")))?;
        
        // Build the HTTP client
        let client = ::reqwest::Client::builder()
            .build()
            .map_err(|e| crate::zyte::error::ZyteError::ReqwestError(e.to_string()))?;
        
        Result::Ok(Self { api_key, client })
    }

    // Sends an extract request to the Zyte API using the provided request payload.
    //
    // Constructs the request with Basic Authentication, sends it, checks status,
    // and deserializes the JSON response. Returns ZyteError on failure.
    pub async fn extract(&self, request_payload: &crate::zyte::request::ZyteRequest) -> Result<crate::zyte::response::ZyteResponse, crate::zyte::error::ZyteError> {
        let url = "https://api.zyte.com/v1/extract";
        
        // Build the Basic Authentication header
        let credentials = format!("{}:", self.api_key);
        let encoded = STANDARD.encode(credentials);
        let auth_value = format!("Basic {encoded}");
        
        // Build and send the request
        let response = self.client.post(url)
            .header(::reqwest::header::AUTHORIZATION, auth_value)
            .json(request_payload)
            .send()
            .await
            .map_err(|e| crate::zyte::error::ZyteError::ReqwestError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Result::Err(
                crate::zyte::error::ZyteError::AuthenticationError(
                    format!("Request failed with status: {}", response.status())
                )
            );
        }
        
        // Deserialize the JSON response
        let mut parsed = response.json::<crate::zyte::response::ZyteResponse>()
            .await
            .map_err(|e| crate::zyte::error::ZyteError::JsonError(e.to_string()))?;
        
                // Decode the http_response_body if present and overwrite the field in the response struct.
        if let Some(encoded_body) = parsed.http_response_body.as_deref() {
            match STANDARD.decode(encoded_body) {
                Ok(decoded_bytes) => {
                    // Overwrite with lossy UTF-8 string representation.
                    // Note: If the body is not UTF-8 (e.g., image), this will contain replacement characters.
                    parsed.http_response_body = Some(::std::string::String::from_utf8_lossy(&decoded_bytes).into_owned());
                    // Optional: Add confirmation log
                    // println!("Successfully decoded and replaced httpResponseBody."); 
                }
                Err(e) => {
                    // Decoding failed. Log a warning and keep the original base64 encoded string.
                    println!("Warning: Failed to decode httpResponseBody from base64: {e}. Keeping original.");
                    // parsed.http_response_body remains unchanged (original base64)
                }
            }
        } else {
            // No httpResponseBody was present in the original response.
             println!("No httpResponseBody present to decode or overwrite.");
        }


        Result::Ok(parsed)
    }

    /// Performs extraction and parses PDF content if present.
    /// Returns Ok(Some(text)) if PDF text is extracted, Ok(None) if no PDF present,
    /// or Err(ZyteError) on failure.
    pub async fn extract_pdf_text(&self, request_payload: &crate::zyte::request::ZyteRequest) -> Result<Option<String>, crate::zyte::error::ZyteError> {
        let response = self.extract(request_payload).await?;
        response.extract_pdf_text()
    }

    /// Performs extraction and cleans HTML boilerplate, returning main content text.
    /// Returns Ok(Some(text)) if HTML content is extracted, Ok(None) if no HTML present,
    /// or Err(ZyteError) on failure.
    pub async fn extract_clean_html(&self, request_payload: &crate::zyte::request::ZyteRequest) -> Result<Option<String>, crate::zyte::error::ZyteError> {
        let response = self.extract(request_payload).await?;
        response.extract_clean_html()
    }
}

#[cfg(test)]
mod tests {
    // Note: Accessing items via crate:: paths as per guidelines (no super:: needed at top level).
    // These tests might require a valid ZYTE_API_KEY environment variable to succeed.

    use crate::ZyteClient;

    #[::tokio::test]
    async fn test_client_new_ok() {
        // Test that client creation succeeds if API key is present.
        // This test implicitly requires ZYTE_API_KEY to be set in the environment.
        ::dotenvy::dotenv().ok(); // Load .env if present
        if ::std::env::var("ZYTE_API_KEY").is_ok() {
            let client_result = ZyteClient::new();
            assert!(client_result.is_ok(), "Client creation failed when ZYTE_API_KEY is set: {:?}", client_result.err());
        } else {
            // If key is not set, we expect an EnvVarError
            let client_result = ZyteClient::new();
            assert!(client_result.is_err());
            match client_result.err().unwrap() {
                crate::zyte::error::ZyteError::EnvVarError(_) => { /* Expected */ }
                e => panic!("Expected EnvVarError, got {:?}", e),
            }
            println!("Skipping client creation test as ZYTE_API_KEY is not set.");
        }
    }

    #[::tokio::test]
    async fn test_basic_extract() {
        // Test a basic extraction. Requires ZYTE_API_KEY.
        dotenvy::dotenv().ok(); // Load .env if present
        if ::std::env::var("ZYTE_API_KEY").is_err() {
            println!("Skipping test_basic_extract as ZYTE_API_KEY is not set.");
            return;
        }

        let client_result = ZyteClient::new();
        assert!(client_result.is_ok(), "Client creation failed unexpectedly");
        let client = client_result.unwrap();

        // Create a simple request
        let request = crate::zyte::request::ZyteRequest {
            url: String::from("http://books.toscrape.com/"),
            http_response_body: Option::Some(true),
            browser_html: Option::None, // Keep request minimal
            screenshot: Option::None,
            screenshot_options: Option::None,
            actions: Option::None,
        };

        // Perform the extraction
        let result = client.extract(&request).await;

        // Assert the result is Ok and contains expected data
        assert!(result.is_ok(), "Extract request failed: {:?}", result.err());
        let response = result.unwrap();

        assert_eq!(response.url, "http://books.toscrape.com/");
        assert_eq!(response.status_code, 200);
        assert!(response.http_response_body.is_some(), "httpResponseBody should be present");
        // Basic check if body is HTML-like (not exhaustive)
        assert!(response.http_response_body.unwrap().contains("<!DOCTYPE html>"));
    }
}
