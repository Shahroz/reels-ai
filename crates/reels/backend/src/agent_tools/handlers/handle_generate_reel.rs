//! Handles the 'generate_reel' agent tool action.
//!
//! This function generates a reel (short video) from a product/service URL or text description
//! with a specified time duration. It fetches product information from URL if provided,
//! then creates an engaging video montage using the video-to-montage cloud function.

use serde_json::{json, Value};
use std::env;

pub async fn handle_generate_reel(
    mut params: crate::agent_tools::tool_params::generate_reel_params::GenerateReelParams,
    pool: &sqlx::PgPool,
    gcs_client: &crate::services::gcs::gcs_client::GCSClient,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let user_id = params
        .user_id
        .ok_or("The user_id should be provided".to_owned())?;
    let organization_id = params.organization_id;
    let tool_name = "generate_reel";

    log::info!(
        "Handling {tool_name} for user {}: prompt='{}', product_url={:?}, time_range={}s",
        user_id,
        params.prompt,
        params.product_url,
        params.time_range_seconds
    );

    // Check credit availability before proceeding
    let credits_to_consume =
        crate::app_constants::credits_constants::CreditsConsumption::GENERATE_CREATIVE;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }

    // Step 1: Fetch product/service information if URL is provided
    let mut enhanced_prompt = params.prompt.clone();
    if let Some(ref product_url) = params.product_url {
        log::info!("Fetching product information from URL: {}", product_url);
        let browse_params = crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams {
            url: product_url.clone(),
            query: format!("Extract key product/service features, benefits, and visual elements that would be useful for creating a promotional reel. Focus on what makes this product/service unique and appealing."),
        };
        
        match crate::agent_tools::handlers::handle_narrativ_browse_with_query::handle_narrativ_browse_with_query(
            browse_params,
            user_id,
            pool,
        ).await {
            Ok((full_resp, _user_resp)) => {
                // Extract summary from the response
                if let Some(content) = full_resp.properties.get("summary") {
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
            Err(e) => {
                log::warn!("Failed to fetch product information from URL {}: {}. Continuing with original prompt.", product_url, e);
            }
        }
    }

    // Step 2: Get video-to-montage cloud function URL from environment
    let montage_function_url = env::var("VIDEO_TO_MONTAGE_FUNCTION_URL")
        .unwrap_or_else(|_| {
            log::warn!("VIDEO_TO_MONTAGE_FUNCTION_URL not set, using default localhost URL");
            "http://localhost:8080".to_string()
        });

    // Step 3: Get GCS bucket name for storing output
    let bucket_name = env::var("GCS_BUCKET_MICROSITES")
        .map_err(|_| "GCS_BUCKET_MICROSITES environment variable not set".to_string())?;

    // Step 4: Generate output GCS URI for the reel
    let reel_id = uuid::Uuid::new_v4();
    let output_gcs_uri = format!("gs://{}/reels/{}/{}.mp4", bucket_name, user_id, reel_id);

    // Step 5: For now, use placeholder assets. In a full implementation, you would:
    // - Search for relevant images/videos in user's assets
    // - Generate images using AI if needed
    // - Use product images from the URL if available
    // For this initial version, we'll create a simplified montage request
    
    // Create a simplified assets array - in production, this would come from:
    // - User's existing assets
    // - AI-generated images based on prompt
    // - Product images extracted from URL
    let assets: Vec<Value> = vec![
        // Placeholder: In production, replace with actual asset GCS URIs
        json!({
            "type": "photo",
            "gcs_uri": format!("gs://{}/reels/{}/placeholder.jpg", bucket_name, user_id)
        })
    ];

    // Step 6: Call video-to-montage cloud function
    log::info!("Calling video-to-montage cloud function: {}", montage_function_url);
    let montage_request = json!({
        "assets": assets,
        "output_gcs_uri": output_gcs_uri,
        "prompt": enhanced_prompt,
        "length": params.time_range_seconds,
        "resolution": [1920, 1080] // Standard reel resolution
    });

    let montage_response = match crate::services::http_request::api_request(
        &montage_function_url,
        reqwest::Method::POST,
        None,
        Some(montage_request),
        None,
    ).await {
        Ok(response) => response,
        Err(e) => {
            let error_msg = format!("Failed to call video-to-montage cloud function: {}", e);
            log::error!("{}", error_msg);
            return std::result::Result::Err(error_msg);
        }
    };

    // Step 7: Extract output path from response
    let output_path = montage_response
        .get("output_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Video-to-montage function did not return output_path".to_string())?;

    // Convert GCS path to HTTPS URL
    let gcs_url = if output_path.starts_with("gs://") {
        let parts: Vec<&str> = output_path[5..].splitn(2, '/').collect();
        if parts.len() == 2 {
            format!("https://storage.googleapis.com/{}/{}", parts[0], parts[1])
        } else {
            return std::result::Result::Err(format!("Invalid GCS path format: {}", output_path));
        }
    } else {
        output_path.to_string()
    };

    let gcs_object_name = format!("reels/{}/{}.mp4", user_id, reel_id);

    // Step 8: Save the generated reel as an asset
    let asset_id = uuid::Uuid::new_v4();
    let asset_name = format!("Reel: {}", params.prompt.chars().take(50).collect::<String>());
    
    match crate::queries::assets::create_asset::create_asset(
        pool,
        asset_id,
        Some(user_id),
        &asset_name,
        "video/mp4",
        &gcs_object_name,
        &gcs_url,
        None, // collection_id
        None, // metadata
        false, // is_public
    ).await {
        Ok(asset) => {
            log::info!("Successfully saved reel asset with ID: {}", asset.id);

            // Step 9: Consume credits
            let credit_params = crate::queries::user_credit_allocation::CreditChangesParams {
                user_id,
                organization_id,
                credit_change: credits_to_consume.amount,
                description: Some(format!("Generated reel: {}", params.prompt)),
            };

            if let Err(e) = crate::queries::user_credit_allocation::deduct_user_credits_with_transaction(
                pool,
                &credit_params,
            ).await {
                log::error!("Failed to deduct credits for reel generation: {}", e);
                // Continue anyway - the reel was created successfully
            }

            // Step 10: Prepare response
            let mut full_response_map = serde_json::Map::new();
            full_response_map.insert("status".to_string(), json!("success"));
            full_response_map.insert("asset_id".to_string(), json!(asset.id.to_string()));
            full_response_map.insert("asset_url".to_string(), json!(asset.url));
            full_response_map.insert("asset_name".to_string(), json!(asset.name));
            full_response_map.insert("duration_seconds".to_string(), json!(params.time_range_seconds));
            
            let full_response_properties = serde_json::Value::Object(full_response_map);
            let user_response_data = Some(json!({
                "asset_id": asset.id.to_string(),
                "asset_url": asset.url,
                "asset_name": asset.name,
                "duration_seconds": params.time_range_seconds,
            }));

            std::result::Result::Ok((
                agentloop::types::full_tool_response::FullToolResponse {
                    properties: full_response_properties,
                },
                agentloop::types::user_tool_response::UserToolResponse {
                    summary: format!(
                        "Successfully generated {} second reel from prompt: '{}'. Asset ID: {}",
                        params.time_range_seconds,
                        params.prompt,
                        asset.id
                    ),
                    data: user_response_data,
                },
            ))
        }
        Err(e) => {
            let error_msg = format!("Failed to save reel asset to database: {}", e);
            log::error!("{}", error_msg);
            std::result::Result::Err(error_msg)
        }
    }
}

