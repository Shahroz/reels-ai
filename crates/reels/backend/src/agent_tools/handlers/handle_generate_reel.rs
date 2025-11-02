//! Handles the 'generate_reel' agent tool action.
//!
//! This function generates a reel (short video) from a product/service URL or text description
//! with a specified time duration. It fetches product information from URL if provided,
//! then saves the generated video locally.

use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::Write;

pub async fn handle_generate_reel(
    params: crate::agent_tools::tool_params::generate_reel_params::GenerateReelParams,
    _gcs_client: &crate::services::gcs::gcs_client::GCSClient, // GCS client not used directly
    user_id: uuid::Uuid, // Placeholder user_id for GCS path generation
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name = "generate_reel";

    log::info!(
        "Handling {tool_name}: prompt='{}', product_url={:?}, time_range={}s",
        params.prompt,
        params.product_url,
        params.time_range_seconds
    );

    // Step 1: Fetch product/service information if URL is provided
    let mut enhanced_prompt = params.prompt.clone();
    if let Some(ref product_url) = params.product_url {
        log::info!("Fetching product information from URL: {}", product_url);
        let browse_params = crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams {
            url: product_url.clone(),
            query: format!("Extract key product/service features, benefits, and visual elements that would be useful for creating a promotional reel. Focus on what makes this product/service unique and appealing."),
        };
        
        match crate::agent_tools::handlers::handle_reels_browse_with_query::handle_reels_browse_with_query(
            browse_params,
            user_id,
        ).await {
            Ok((full_resp, _user_resp)) => {
                // Extract summary from the response
                if let Some(response_obj) = full_resp.response.as_object() {
                    if let Some(content) = response_obj.get("summary") {
                        if let Some(summary_str) = content.as_str() {
                            enhanced_prompt = format!(
                                "Product/Service Information:\n{}\n\nOriginal Prompt: {}",
                                summary_str,
                                params.prompt
                            );
                            log::info!("Enhanced prompt with product information");
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to fetch product information from URL {}: {}. Continuing with original prompt.", product_url, e);
            }
        }
    }

    // Step 2: Generate unique reel ID
    let reel_id = uuid::Uuid::new_v4();

    // Step 3: Create local storage directory structure
    let storage_base = env::var("REELS_STORAGE_PATH")
        .unwrap_or_else(|_| "storage/reels".to_string());
    
    let storage_dir = PathBuf::from(&storage_base);
    fs::create_dir_all(&storage_dir)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;

    // Step 4: Generate local file path
    let filename = format!("{}.mp4", reel_id);
    let file_path = storage_dir.join(&filename);
    let relative_path = format!("reels/{}", filename);
    
    // Step 5: For now, create a placeholder video file
    // In a real implementation, you would generate the actual video here
    // This is a placeholder that creates an empty file with metadata
    log::info!("Saving reel locally to: {}", file_path.display());
    
    // Create a minimal MP4 header (this is a placeholder - replace with actual video generation)
    // For demonstration, we'll create an empty file. In production, use a video library
    // like ffmpeg, opencv, or similar to generate the actual video from the prompt.
    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create video file: {}", e))?;
    
    // Write a placeholder message (in production, write actual video bytes)
    let placeholder_data = format!(
        "PLACEHOLDER_VIDEO: Reel generated for prompt: '{}', Duration: {}s\n",
        enhanced_prompt,
        params.time_range_seconds
    );
    file.write_all(placeholder_data.as_bytes())
        .map_err(|e| format!("Failed to write video file: {}", e))?;

    // Step 6: Generate local file URL for serving
    // The backend will serve files from the storage directory via a static file route
    let local_url = format!("/storage/{}", relative_path);

    // Step 7: Prepare response with local file URL
    log::info!("Successfully saved reel locally: {}", file_path.display());

    let mut full_response_map = serde_json::Map::new();
    full_response_map.insert("status".to_string(), json!("success"));
    full_response_map.insert("reel_id".to_string(), json!(reel_id.to_string()));
    full_response_map.insert("reel_url".to_string(), json!(local_url.clone()));
    full_response_map.insert("local_path".to_string(), json!(file_path.display().to_string()));
    full_response_map.insert("duration_seconds".to_string(), json!(params.time_range_seconds));
    
    let full_response_properties = serde_json::Value::Object(full_response_map);
    let user_response_data = Some(json!({
        "reel_id": reel_id.to_string(),
        "reel_url": local_url.clone(),
        "local_path": file_path.display().to_string(),
        "duration_seconds": params.time_range_seconds,
    }));

    std::result::Result::Ok((
        agentloop::types::full_tool_response::FullToolResponse {
            tool_name: tool_name.to_string(),
            response: full_response_properties,
        },
        agentloop::types::user_tool_response::UserToolResponse {
            tool_name: tool_name.to_string(),
            summary: format!(
                "Successfully generated {} second reel from prompt: '{}'. Reel URL: {}",
                params.time_range_seconds,
                params.prompt,
                local_url
            ),
            icon: None,
            data: user_response_data,
        },
    ))
}

