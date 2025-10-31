use std::env;
use reqwest::Client;
use serde_json::{json, Value};
use anyhow::anyhow;
use tracing::instrument;

/// Takes a screenshot of the HTML snippet using the LIGHTHOUSE_API.
/// Returns a base64 encoded screenshot.
#[instrument(skip(html))]
pub async fn screenshot_html(html: &str, full_page: bool) -> anyhow::Result<String> {
    let lighthouse_url = env::var("LIGHTHOUSE_URL").map_err(|_| anyhow::anyhow!("LIGHTHOUSE_URL must be set"))?;
    let lighthouse_api_key = env::var("LIGHTHOUSE_API_KEY").map_err(|_| anyhow::anyhow!("LIGHTHOUSE_API_KEY must be set"))?;

    let client = Client::new();
    let request_body = json!({ "html": html, "full_page": full_page });
    let response: Value = client
        .post(&format!("{}/screenshot", lighthouse_url))
        .header("Authorization", format!("Bearer {}", lighthouse_api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?
        .json()
        .await?;

    let screenshot = response
        .as_object()
        .ok_or(anyhow!("Cannot parse response as object"))?
        .get("screenshot")
        .ok_or(anyhow!("Missing screenshot field"))?
        .as_str()
        .ok_or(anyhow!("Screenshot field is not a string"))?;
    Ok(screenshot.to_string())
}