use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use tracing::instrument;

/// Makes an HTTP request to the specified endpoint with the given parameters.
///
/// # Arguments
///
/// * `endpoint` - The full API URL to send the request to
/// * `method` - The HTTP method (GET, POST, etc.)
/// * `params` - Optional query parameters as key-value pairs
/// * `body` - Optional JSON body
/// * `headers` - Optional HTTP headers
///
/// # Returns
///
/// Returns a Result containing the parsed JSON response or an error
#[instrument(skip(method, params, body, headers))]
pub async fn api_request(
    endpoint: &str,
    method: reqwest::Method,
    params: Option<&[(&str, &str)]>,
    body: Option<Value>,
    headers: Option<HeaderMap>,
) -> Result<Value, Box<dyn Error>> {
    // Create a new client
    let client = reqwest::Client::new();

    // Build the request
    let mut request_builder = client.request(method, endpoint);

    // Add query parameters if provided
    if let Some(params) = params {
        request_builder = request_builder.query(params);
    }

    // Add body if provided
    if let Some(body) = body {
        //let body_str = body.to_string();
        request_builder = request_builder.json(&json!(&body));
    }

    // Set up headers
    let mut final_headers = HeaderMap::new();

    // Add default headers
    final_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    final_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    // Merge with custom headers if provided
    if let Some(custom_headers) = headers {
        for (key, value) in custom_headers.iter() {
            final_headers.insert(key, value.clone());
        }
    }

    request_builder = request_builder.headers(final_headers);

    // Send the request and handle the response
    let response = request_builder.send().await?;

    // Check if the response status is successful
    if !response.status().is_success() {
        let status = response.status();
        let err = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read body".to_string());
        log::error!("HTTP request failed with status: {status} ({err})");
        return Err(format!("HTTP request failed with status: {status}").into());
    }

    // Parse the JSON response
    let json_response: Value = response.json().await?;
    Ok(json_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_api_request_success() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up the mock
        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"success": true})),
            )
            .mount(&mock_server)
            .await;

        // Make the request
        let response = api_request(
            &format!("{}/test", mock_server.uri()),
            reqwest::Method::POST,
            Some(&[("param", "value")]),
            Some(serde_json::json!({"key": "value"})),
            None,
        )
        .await;

        // Check the response
        assert!(response.is_ok());
        let json = response.unwrap();
        assert_eq!(json["success"], true);
    }

    #[tokio::test]
    async fn test_api_request_error() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up the mock to return an error
        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        // Make the request
        let response = api_request(
            &format!("{}/test", mock_server.uri()),
            reqwest::Method::POST,
            None,
            None,
            None,
        )
        .await;

        // Check that we got an error
        assert!(response.is_err());
    }
}