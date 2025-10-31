use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use thiserror::Error;

const WEBFLOW_API_BASE_URL: &str = "https://api.webflow.com/v2";

#[derive(Debug, Error)]
pub enum WebflowError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde JSON error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Webflow API error: {status} - {body}")]
    ApiError {
        status: reqwest::StatusCode,
        body: String,
    },
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateItemPayload {
    field_data: HashMap<String, Value>,
    is_draft: bool,
    is_archived: bool,
}

// Basic structure for the response, adjust as needed based on actual Webflow response
#[derive(Deserialize, Debug, PartialEq)]
pub struct ItemResponse {
    id: String,
    // Add other fields returned by Webflow API as needed
    #[serde(rename = "fieldData")]
    field_data: HashMap<String, Value>,
    #[serde(rename = "lastPublished")]
    last_published: Option<String>,
    #[serde(rename = "lastUpdated")]
    last_updated: Option<String>,
    #[serde(rename = "createdOn")]
    created_on: Option<String>,
    #[serde(rename = "isArchived")]
    is_archived: bool,
    #[serde(rename = "isDraft")]
    is_draft: bool,
}


#[derive(Clone)]
pub struct WebflowClient {
    client: reqwest::Client,
    api_token: String,
    api_version: String,
}

impl WebflowClient {
    pub fn new() -> Result<Self, WebflowError> {
        let api_token = env::var("WEBFLOW_API_TOKEN")
            .map_err(|_| WebflowError::MissingEnvVar("WEBFLOW_API_TOKEN".to_string()))?;
        let api_version = env::var("WEBFLOW_API_VERSION")
            .map_err(|_| WebflowError::MissingEnvVar("WEBFLOW_API_VERSION".to_string()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_token,
            api_version,
        })
    }

    /// Create a new item in an existing CMS Collection (v2 Data API).
    /// field_data must include at least 'name' and 'slug'.
    pub async fn create_collection_item(
        &self,
        collection_id: &str,
        field_data: HashMap<String, Value>,
    ) -> Result<ItemResponse, WebflowError> {
        let url = format!("{WEBFLOW_API_BASE_URL}/collections/{collection_id}/items");

        let is_draft = field_data.get("_draft").and_then(|v| v.as_bool()).unwrap_or(false);
        let is_archived = field_data.get("_archived").and_then(|v| v.as_bool()).unwrap_or(false);

        // Remove _draft and _archived from field_data if they exist, as they are top-level
        let mut clean_field_data = field_data.clone();
        clean_field_data.remove("_draft");
        clean_field_data.remove("_archived");

        let payload = CreateItemPayload {
            field_data: clean_field_data,
            is_draft,
            is_archived,
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&self.api_token)
            .header("accept-version", &self.api_version)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let item_response = response.json::<ItemResponse>().await?;
            Ok(item_response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            Err(WebflowError::ApiError { status, body })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Use super::* to import from the parent module (client.rs)
    use dotenvy::dotenv;
    use serde_json::json;

    // Helper function to get a configured client for integration tests
    fn setup_integration_client() -> Result<WebflowClient, WebflowError> {
        dotenv().ok(); // Load .env file for credentials
        WebflowClient::new()
    }

    // Mark as ignored by default, as this hits the live API
    #[tokio::test]
    #[ignore]
    async fn test_integration_create_collection_item_success() {
        let client = setup_integration_client()
            .expect("Failed to create client. Ensure WEBFLOW_API_TOKEN and WEBFLOW_API_VERSION are set in .env or environment.");

        // Use a known *real* collection ID from your Webflow account
        let collection_id = "6810d80fe128cbce9928047f"; // <---- CHANGE THIS
        let item_name = "Integration Test Item";
        // Use a unique slug to avoid conflicts
        let item_slug = format!("integration-test-item-{}", chrono::Utc::now().timestamp_millis());

        let mut field_data: HashMap<String, Value> = HashMap::new();
        field_data.insert("name".to_string(), json!(item_name));
        field_data.insert("slug".to_string(), json!(item_slug));
        // Add any other required fields for your collection
        field_data.insert("header".to_string(), json!("Some Value header"));
        field_data.insert("sub-header".to_string(), json!("Some Value sub-header"));

        // Call the actual client method
        let result = client.create_collection_item(collection_id, field_data).await;

        // Assert that the call was successful
        assert!(result.is_ok(), "API call failed: {:?}", result.err());

        if let Ok(item) = result {
            println!("Successfully created item: {:?}", item);
            assert_eq!(item.field_data.get("name").unwrap(), &json!(item_name));
            assert_eq!(item.field_data.get("slug").unwrap(), &json!(item_slug));
            assert!(!item.is_draft);
            assert!(!item.is_archived);
            // Add assertions for other fields if necessary

            // TODO: Consider adding cleanup logic to delete the created item
        }
    }

    // Mark as ignored by default
    #[tokio::test]
    #[ignore]
    async fn test_integration_create_collection_item_with_draft() {
        let client = setup_integration_client()
            .expect("Failed to create client. Ensure WEBFLOW_API_TOKEN and WEBFLOW_API_VERSION are set in .env or environment.");

        let collection_id = "YOUR_REAL_COLLECTION_ID"; // <---- CHANGE THIS
        let item_name = "Integration Test Draft Item";
        let item_slug = format!("integration-test-draft-item-{}", chrono::Utc::now().timestamp_millis());

        let mut field_data: HashMap<String, Value> = HashMap::new();
        field_data.insert("name".to_string(), json!(item_name));
        field_data.insert("slug".to_string(), json!(item_slug));
        field_data.insert("_draft".to_string(), json!(true)); // Mark as draft

        let result = client.create_collection_item(collection_id, field_data).await;

        assert!(result.is_ok(), "API call failed: {:?}", result.err());

        if let Ok(item) = result {
            println!("Successfully created draft item: {:?}", item);
            assert_eq!(item.field_data.get("name").unwrap(), &json!(item_name));
            assert_eq!(item.field_data.get("slug").unwrap(), &json!(item_slug));
            assert!(item.is_draft); // Check draft status
            assert!(!item.is_archived);

            // TODO: Consider adding cleanup logic to delete the created item
        }
    }

    // Mark as ignored by default
    #[tokio::test]
    #[ignore]
    async fn test_integration_create_collection_item_api_error() {
        let client = setup_integration_client()
            .expect("Failed to create client. Ensure WEBFLOW_API_TOKEN and WEBFLOW_API_VERSION are set in .env or environment.");

        // Use an invalid collection ID to intentionally cause an error
        let collection_id = "invalid-collection-id-12345";
        let item_name = "Integration Test Error Item";
        let item_slug = "integration-test-error-item";

        let mut field_data: HashMap<String, Value> = HashMap::new();
        field_data.insert("name".to_string(), json!(item_name));
        field_data.insert("slug".to_string(), json!(item_slug));

        let result = client.create_collection_item(collection_id, field_data).await;

        // Assert that the call resulted in an error
        assert!(result.is_err(), "API call unexpectedly succeeded");

        if let Err(err) = result {
            println!("Received expected error: {:?}", err);
            match err {
                WebflowError::ApiError { status, body } => {
                    // Expecting a 404 Not Found or similar error from Webflow
                    assert!(status.is_client_error() || status.is_server_error());
                    // You could add more specific checks on the status code or body if needed
                    // e.g., assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                    // assert!(body.contains("NotFound"));
                }
                _ => panic!("Expected WebflowError::ApiError, but got a different error type: {:?}", err),
            }
        }
    }
} 
