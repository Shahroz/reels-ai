//! Handles video generation requests with Google's Veo 3 API.
//!
//! This module provides functionality to generate videos using the Veo 3 API
//! with long-running operations. It follows the same patterns as `completion_conversation.rs`
//! for API key handling, client creation, and error handling.
//! Uses fully qualified paths for all dependencies and adheres to coding guidelines.

use std::time::Duration;

/// Asynchronously generates a video using the Gemini Veo 3 API with the specified prompt,
/// then returns the video bytes.
///
/// This function initiates a long-running operation, polls for completion,
/// and downloads the generated video.
pub async fn generate_veo3_video(
    prompt: String,
    model: Option<String>,
    duration_seconds: Option<u32>,
) -> std::result::Result<Vec<u8>, std::boxed::Box<dyn std::error::Error>> {
    // Create an HTTP client with extended timeout for long-running operations
    let client = reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_secs(600)) // 10 minutes for video generation
        .build()
        .map_err(|e| {
            log::error!("Failed to build HTTP client for Veo 3: {}", e);
            e
        })?;

    // Retrieve the API key
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key.trim_matches('"').to_string(),
        Err(e) => {
            log::error!("GEMINI_API_KEY not found: {}", e);
            return std::result::Result::Err(std::boxed::Box::new(e));
        }
    };

    // Use the specified model or default to veo-3.1-generate-preview
    let model_id = model.unwrap_or_else(|| "veo-3.1-generate-preview".to_string());
    let base_url = "https://generativelanguage.googleapis.com/v1beta";
    let generate_url = format!("{}/models/{}:predictLongRunning", base_url, model_id);

    // Build request body
    // The Veo 3.1 API expects:
    // - instances: array with prompt
    // - parameters: object with durationSeconds and other optional params
    let instance_obj = serde_json::json!({
        "prompt": prompt
    });
    
    let mut request_body = serde_json::json!({
        "instances": [instance_obj]
    });
    
    // Add parameters object with durationSeconds if provided (API expects camelCase)
    if let Some(duration) = duration_seconds {
        if let Some(body_obj) = request_body.as_object_mut() {
            let mut params_obj = serde_json::Map::new();
            params_obj.insert("durationSeconds".to_string(), serde_json::json!(duration));
            body_obj.insert("parameters".to_string(), serde_json::Value::Object(params_obj));
        }
    }

    // Send the POST request with retry logic
    let mut last_error: Option<std::string::String> = None;
    let max_retries = 5;

    let operation_name = {
        let mut result: std::result::Result<String, String> = Err("Initialization".to_string());
        
        for attempt in 0..max_retries {
            if attempt > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }

            log::info!("Sending video generation request to Veo 3 API (attempt {})", attempt + 1);
            log::debug!("Request body: {}", serde_json::to_string(&request_body).unwrap_or_else(|_| "Failed to serialize".to_string()));

            let request_builder = client
                .post(&generate_url)
                .header("x-goog-api-key", &api_key)
                .header("Content-Type", "application/json")
                .json(&request_body);

            let response_result = request_builder.send().await;

            match response_result {
                Ok(response) => {
                    match response.error_for_status() {
                        Ok(successful_response) => {
                            match successful_response.json::<serde_json::Value>().await {
                                Ok(response_json) => {
                                    if let Some(name) = response_json.get("name").and_then(|n| n.as_str()) {
                                        log::info!("Video generation operation started: {}", name);
                                        result = Ok(name.to_string());
                                        break;
                                    } else {
                                        let err_msg = "Operation response missing 'name' field".to_string();
                                        log::error!("{}", err_msg);
                                        result = Err(err_msg);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    let err_msg = format!("Failed to deserialize operation response JSON: {}", e);
                                    log::error!("{}", err_msg);
                                    if attempt < max_retries - 1 {
                                        last_error = Some(err_msg.clone());
                                        continue;
                                    }
                                    result = Err(err_msg);
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            // For 400 errors, try to extract response body for better debugging
                            if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
                                let error_text = err.to_string();
                                let err_msg = format!(
                                    "Attempt {}: HTTP error in video generation: {}. Request body: {}",
                                    attempt + 1,
                                    error_text,
                                    serde_json::to_string(&request_body).unwrap_or_else(|_| "Failed to serialize".to_string())
                                );
                                log::error!("{}", err_msg);
                                result = Err(err_msg);
                                break;
                            }
                            if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                                if attempt < max_retries - 1 {
                                    let wait_time_ms = std::cmp::max(1000, 100 * 2u64.pow(attempt as u32));
                                    tokio::time::sleep(tokio::time::Duration::from_millis(wait_time_ms)).await;
                                    let warn_msg = format!("Attempt {}: Rate limited (429) for video generation. Retrying after {}ms. Error: {}", attempt + 1, wait_time_ms, err);
                                    log::warn!("{}", warn_msg);
                                    last_error = Some(warn_msg);
                                    continue;
                                }
                                let err_msg = format!("Attempt {}: Rate limited (429) for video generation. Max retries reached. Error: {}", attempt + 1, err);
                                log::error!("{}", err_msg);
                                result = Err(err_msg);
                                break;
                            }
                            let err_msg = format!("Attempt {}: HTTP error in video generation: {}", attempt + 1, err);
                            log::error!("{}", err_msg);
                            result = Err(err_msg);
                            break;
                        }
                    }
                }
                Err(err) => {
                    if attempt < max_retries - 1 {
                        let wait_time_ms = std::cmp::max(1000, 200 * 2u64.pow(attempt as u32));
                        tokio::time::sleep(tokio::time::Duration::from_millis(wait_time_ms)).await;
                        let warn_msg = format!("Attempt {}: Network error in video generation. Retrying after {}ms. Error: {}", attempt + 1, wait_time_ms, err);
                        log::warn!("{}", warn_msg);
                        last_error = Some(warn_msg);
                        continue;
                    }
                    let err_msg = format!("Attempt {}: Network error in video generation. Max retries reached. Error: {}", attempt + 1, err);
                    log::error!("{}", err_msg);
                    result = Err(err_msg);
                    break;
                }
            }
        }

        match result {
            Ok(name) => Ok(name),
            Err(e) => Err(e),
        }
    };
    
    let operation_name = operation_name.map_err(|e| {
        let error_msg = if e == "Initialization" {
            last_error.unwrap_or_else(|| {
                "Video generation request failed after multiple retries with no specific error recorded".to_string()
            })
        } else {
            e
        };
        std::boxed::Box::<dyn std::error::Error>::from(error_msg)
    })?;

    // Poll operation status until complete
    let poll_url = format!("{}/{}", base_url, operation_name);
    let mut operation_done = false;
    let mut poll_count = 0;
    const MAX_POLLS: u32 = 120; // Maximum 20 minutes (120 * 10 seconds)

    while !operation_done && poll_count < MAX_POLLS {
        tokio::time::sleep(Duration::from_secs(10)).await;
        poll_count += 1;

        log::info!("Polling operation status (attempt {})...", poll_count);

        let status_response = client
            .get(&poll_url)
            .header("x-goog-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to poll operation status: {}", e))?;

        if !status_response.status().is_success() {
            let status = status_response.status();
            let error_text = status_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return std::result::Result::Err(std::boxed::Box::from(format!(
                "Operation status check failed with status {}: {}",
                status, error_text
            )));
        }

        let status_json: serde_json::Value = status_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse operation status: {}", e))?;

        operation_done = status_json
            .get("done")
            .and_then(|d| d.as_bool())
            .unwrap_or(false);

        // Check for operation errors
        if let Some(error) = status_json.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return std::result::Result::Err(std::boxed::Box::from(format!(
                "Video generation operation failed: {}",
                error_message
            )));
        }

        if operation_done {
            log::info!("Video generation completed");

            // Extract video URI from response
            let video_uri = status_json
                .get("response")
                .and_then(|r| r.get("generateVideoResponse"))
                .and_then(|gvr| gvr.get("generatedSamples"))
                .and_then(|samples| samples.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|sample| sample.get("video"))
                .and_then(|video| video.get("uri"))
                .and_then(|uri| uri.as_str())
                .ok_or_else(|| "Video URI not found in operation response".to_string())?;

            log::info!("Downloading video from URI: {}", video_uri);

            // Download the generated video with retry logic
            let mut download_attempt = 0;
            const MAX_DOWNLOAD_RETRIES: u32 = 3;

            loop {
                let video_response = client
                    .get(video_uri)
                    .header("x-goog-api-key", &api_key)
                    .send()
                    .await;

                match video_response {
                    Ok(response) => {
                        if !response.status().is_success() {
                            let status = response.status();
                            if download_attempt < MAX_DOWNLOAD_RETRIES - 1 {
                                download_attempt += 1;
                                let wait_time = 1000 * download_attempt as u64;
                                log::warn!("Video download failed with status {}. Retrying after {}ms...", status, wait_time);
                                tokio::time::sleep(Duration::from_millis(wait_time)).await;
                                continue;
                            }
                            return std::result::Result::Err(std::boxed::Box::from(format!(
                                "Video download failed with status: {}",
                                status
                            )));
                        }

                        let video_bytes = response
                            .bytes()
                            .await
                            .map_err(|e| format!("Failed to read video bytes: {}", e))?;

                        log::info!("Successfully downloaded video ({} bytes)", video_bytes.len());
                        return std::result::Result::Ok(video_bytes.to_vec());
                    }
                    Err(e) => {
                        if download_attempt < MAX_DOWNLOAD_RETRIES - 1 {
                            download_attempt += 1;
                            let wait_time = 1000 * download_attempt as u64;
                            log::warn!("Video download network error: {}. Retrying after {}ms...", e, wait_time);
                            tokio::time::sleep(Duration::from_millis(wait_time)).await;
                            continue;
                        }
                        return std::result::Result::Err(std::boxed::Box::from(format!(
                            "Failed to download video: {}",
                            e
                        )));
                    }
                }
            }
        }
    }

    if !operation_done {
        return std::result::Result::Err(std::boxed::Box::from(format!(
            "Video generation timed out after {} polling attempts",
            poll_count
        )));
    }

    // This should never be reached, but Rust requires it
    std::result::Result::Err(std::boxed::Box::from("Unexpected end of video generation flow".to_string()))
}

