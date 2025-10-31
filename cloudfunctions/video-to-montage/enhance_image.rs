//! Implementation for editing an image with Gemini for the GenericMapNode.
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use std::path::Path;

use crate::gcs_utils::gcs_file::GCSFile;
use crate::integrations::gcp_auth::get_gcp_authn_token;
use crate::value::NodeInnerValue;
use llm::vendors::gemini::gemini_model::GeminiModel;
use crate::integrations::media_processing::image_resize_to_max_1080_png::resize_to_max_1080_png_or_passthrough;

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    candidates: Vec<Candidate>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    content: Content,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Content {
    parts: Vec<Part>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Part {
    inline_data: Option<InlineData>,
    text: Option<String>,
}
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    data: String,
}

/// Edits an image from a GCSFile using the Gemini API.
///
/// Takes a `NodeInnerValue`, and if it's a `GCSFile`, edits the image
/// using the provided prompt and model. Returns a new `NodeInnerValue` with the
/// edited image data. If the input is not a `GCSFile`, it returns an error.
pub async fn edit_with_gemini_image(
    source_value: &NodeInnerValue,
    prompt: &str,
    model: &GeminiModel,
) -> Result<NodeInnerValue> {
    if let NodeInnerValue::GCSFile(gcs_file) = source_value {
        let project_id = std::env::var("GCP_PROJECT_ID").context("GCP_PROJECT_ID not set")?;

        let processed_image = resize_to_max_1080_png_or_passthrough(gcs_file.data.clone());

        let encoded_image =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &processed_image);

        let request_body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [
                    {
                        "inlineData": {
                            "mimeType": gcs_file.content_type,
                            "data": encoded_image
                        }
                    },
                    {"text": prompt}
                ]
            }],
            "generationConfig": {
                "temperature": 1,
                "maxOutputTokens": 32768,
                "responseModalities": ["TEXT", "IMAGE"],
                "topP": 0.95
            },
            "safetySettings": [
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_IMAGE_HATE", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_IMAGE_DANGEROUS_CONTENT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_IMAGE_HARASSMENT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_IMAGE_SEXUALLY_EXPLICIT", "threshold": "OFF" }
            ]
        });

        let model_id = model.to_string();
        let url = format!(
            "https://aiplatform.googleapis.com/v1/projects/{}/locations/global/publishers/google/models/{}:streamGenerateContent",
            project_id, model_id
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to build reqwest client")?;

        let response = async {
            let mut last_error: Option<anyhow::Error> = None;
            for attempt in 1..=3 {
                let token = get_gcp_authn_token().await.context("Failed to get GCP token")?;
                match client
                    .post(&url)
                    .bearer_auth(token)
                    .json(&request_body)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            return Ok(resp);
                        } else {
                            let status = resp.status();
                            let error_body = resp
                                .text()
                                .await
                                .unwrap_or_else(|_| "Could not read error body".to_string());
                            last_error = Some(anyhow!(
                                "Attempt {} failed with status {}: {}",
                                attempt,
                                status,
                                error_body
                            ));
                        }
                    }
                    Err(e) => {
                        last_error =
                            Some(anyhow!("Attempt {} failed with network error: {}", attempt, e));
                    }
                }
                if attempt < 3 {
                    log::warn!("Retrying gemini request in 2 seconds... {:?}", last_error);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
            Err(last_error.unwrap_or_else(|| anyhow!("Request failed after 3 attempts")))
        }
            .await?;

        let api_responses = response
            .json::<Vec<ApiResponse>>()
            .await
            .context("Failed to parse Gemini response")?;

        let image_data_part = api_responses
            .into_iter()
            .flat_map(|r| r.candidates)
            .flat_map(|c| c.content.parts)
            .find(|p| p.inline_data.is_some())
            .and_then(|p| p.inline_data)
            .ok_or_else(|| anyhow!("Gemini response did not contain image data"))?;

        let retouched_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            image_data_part.data,
        )
            .map_err(|e| anyhow!("Failed to decode base64 image: {}", e))?;

        let path = Path::new(&gcs_file.object);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        let parent = path.parent().and_then(|p| p.to_str()).unwrap_or_default();

        let new_extension = "png"; // Gemini editing often returns png

        // Generate 6-letter random string
        let random_string: String =
            rand::thread_rng().sample_iter(&Alphanumeric).take(6).map(char::from).collect();

        // Generate timestamp (Unix timestamp)
        let timestamp = Utc::now().timestamp();

        let new_object_name = if parent.is_empty() {
            format!("{}_edit_{}_{}.{}", stem, random_string, timestamp, new_extension)
        } else {
            format!("{}/{}_edit_{}_{}.{}", parent, stem, random_string, timestamp, new_extension)
        };

        // Determine content type from the new extension.
        let content_type = "image/png".to_string();

        let new_gcs_file = GCSFile {
            bucket: gcs_file.bucket.clone(),
            object: new_object_name,
            content_type,
            data: retouched_bytes,
        };

        Ok(NodeInnerValue::GCSFile(new_gcs_file))
    } else {
        Err(anyhow::anyhow!(
            "Unsupported input type for editing. Expected GCSFile, got a different NodeInnerValue variant."
        ))
    }
}
