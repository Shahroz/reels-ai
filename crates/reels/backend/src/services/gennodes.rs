use crate::services::http_request::api_request;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::env;
use tracing::instrument;

#[derive(Serialize, Deserialize)]
struct JwtPayload {
    role: String,
    tenant_id: String,
}

#[instrument]
pub async fn get_default_headers_with_authorization(
    user_id: &str,
) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let token = get_jwt_token(user_id).await?;

    let mut final_headers = HeaderMap::new();
    let bearer = format!("Bearer {token}");
    final_headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer)?);
    final_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    Ok(final_headers)
}

/// Gets a JWT token for Gennodes API authentication
///
/// # Arguments
///
/// * `req` - The HTTP request containing the user session
///
/// # Returns
///
/// Returns a Result containing the JWT token string or an error
#[instrument]
pub async fn get_jwt_token(user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret = env::var("JWT_GENNODES_SECRET").unwrap_or_default();
    let expiry = env::var("JWT_GENNODES_TOKEN_EXPIRY").unwrap_or_default();

    let payload = json!({
        "role": "app",
        "tenant_id": user_id
    });

    crate::utils::jwt::generate_jwt_token(&secret, &expiry, &payload)
}

/// Makes a smart-fill-json request to the Gennodes API
///
/// # Arguments
///
/// * `req` - The HTTP request containing the request body and parameters
///
/// # Returns
///
/// Returns a Result containing the API response or an error
#[instrument(skip(payload))]
pub async fn smart_fill_json(
    user_id: &str,
    payload: Option<Value>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let base_url = env::var("GENNODES_BASE_API_URL")?;

    let final_headers = get_default_headers_with_authorization(user_id).await?;

    let response = api_request(
        &format!("{base_url}/smart-fill-json"),
        Method::POST,
        None,
        payload,
        Some(final_headers),
    )
    .await?;

    Ok(response)
}

/// Creates a workflow from a template using the Gennodes API
///
/// # Arguments
///
/// * `req` - The HTTP request containing the request body and parameters
///
/// # Returns
///
/// Returns a Result containing the API response or an error
#[instrument(skip(payload))]
pub async fn workflow_from_template(
    user_id: &str,
    payload: Option<Value>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let base_url = env::var("GENNODES_BASE_API_URL")?;

    let final_headers = get_default_headers_with_authorization(user_id).await?;

    let response = api_request(
        &format!("{base_url}/workflow-from-template"),
        Method::POST,
        None,
        payload,
        Some(final_headers),
    )
    .await?;

    Ok(response)
}

/// Runs a workflow using the Gennodes API
///
/// # Arguments
///
/// * `req` - The HTTP request containing the request body and parameters
///
/// # Returns
///
/// Returns a Result containing the API response or an error
#[instrument(skip(payload))]
pub async fn run_workflow(
    user_id: &str,
    payload: Option<Value>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let base_url = env::var("GENNODES_BASE_API_URL")?;

    let final_headers = get_default_headers_with_authorization(user_id).await?;

    let response = api_request(
        &format!("{base_url}/workflow/run"),
        Method::POST,
        None,
        payload,
        Some(final_headers),
    )
    .await?;

    Ok(response)
}
