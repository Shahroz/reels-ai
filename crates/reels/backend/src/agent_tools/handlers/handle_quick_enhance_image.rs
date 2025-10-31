//! Implements the logic for the Quick Enhance Image tool.
//!
//! This function takes an image (base64 encoded) and an enhancement prompt,
//! uses various image enhancement models to enhance the image based on the prompt,
//! and returns the enhanced image data directly without creating asset records.
//! Currently uses the Gemini 2.5 Flash Image Preview model which is the latest for image generation.
//! 
//! Configuration via required environment variables:
//! - GEMINI_API_KEY: API key for Gemini API access (required)
//! 
//! Adheres to Narrativ and AgentLoop coding standards with one item per file structure.

use llm::vendors::gemini::{
    completion_conversation::generate_gemini_conversation_response,
    content::Content,
    gemini_model::GeminiModel,
    part::Part,
    inline_data::InlineData,
};
use crate::db::assets::Asset;

/// Handles the Quick Enhance Image tool execution.
///
/// Takes an image and enhancement prompt, processes it through various image enhancement models,
/// and returns the enhanced image data directly without creating asset records.
/// Currently uses Gemini 2.5 Flash Image Preview model which is the latest for image generation.
///
/// # Arguments
///
/// * `params` - `crate::agent_tools::tool_params::quick_enhance_image_params::QuickEnhanceImageParams` containing image and prompt details.
///
/// # Returns
///
/// A `Result` containing a tuple of
/// `(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse)`
/// on success, or an error `String` on failure.
pub async fn handle_quick_enhance_image(
    params: crate::agent_tools::tool_params::quick_enhance_image_params::QuickEnhanceImageParams,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let tool_name = "quick_enhance_image".to_string();
    
    // Validate enhancement prompt
    if params.enhancement_prompt.trim().is_empty() {
        return std::result::Result::Err("Enhancement prompt is required and cannot be empty".to_string());
    }
    
    // Validate that we have either image or asset
    if params.image_data.is_none() && params.asset_id.is_none() {
        return std::result::Result::Err("Either image or asset is required".to_string());
    }
    
    // Get API key
    let _api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY environment variable is required".to_string())?;

    if params.image_data.is_some() && params.asset_id.is_some() {
        return std::result::Result::Err("Provide either image or asset, not both".to_string());
    }
    
    // Check credit availability before proceeding
    let user_id = params.user_id.ok_or("User id should be provided".to_string())?;
    let organization_id = params.organization_id;
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::QUICK_ENHANCE_IMAGE;
    let pool: &sqlx::PgPool = crate::db_pool::GLOBAL_POOL.get_ref();
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }

    // Get image bytes from either source
    let image_bytes = if let Some(image_data) = &params.image_data {
        // Parse the base64 image data
        let image_data = if image_data.starts_with("data:") {
            // Extract base64 data from data URL
            let parts: Vec<&str> = image_data.split(',').collect();
            if parts.len() != 2 {
                return std::result::Result::Err("Invalid data URL format".to_string());
            }
            parts[1].to_string()
        } else {
            image_data.clone()
        };

        // Decode base64 to get image bytes
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &image_data) {
            std::result::Result::Ok(bytes) => bytes,
            std::result::Result::Err(e) => {
                return std::result::Result::Err(format!("Failed to decode base64 image data: {e}"));
            }
        }
    } else if let Some(asset_id) = params.asset_id {
        // Fetch asset from database and get image from GCS

        // Fetch asset from database with ownership validation
        let asset = sqlx::query_as!(
            Asset,
            "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = $1 AND (user_id = $2 OR is_public = true)",
            asset_id,
            user_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to fetch asset: {e}"))?
        .ok_or_else(|| "Asset not found or access denied".to_string())?;

        // Parse GCS URL to get bucket and object
        let gcs_url = asset.url;
        let (bucket, object) = crate::services::gcs::parse_gcs_url::parse_gcs_url(&gcs_url)
            .map_err(|e| format!("Failed to parse GCS URL: {e}"))?;

        // Get GCS client
        let gcs = crate::services::gcs::gcs_client::GCSClient::new();

        // Download image from GCS
        gcs.download_object_as_bytes(&bucket, &object).await
            .map_err(|e| format!("Failed to download image from GCS: {e}"))?
    } else {
        return std::result::Result::Err("No valid image source provided".to_string());
    };

    // Determine MIME type from image bytes or use provided output MIME type
    let mime_type = if let Some(output_mime_type) = &params.output_mime_type {
        // Use the provided output MIME type
        output_mime_type.as_str()
    } else {
        // Auto-detect MIME type from image bytes
        if image_bytes.len() >= 4 {
            match &image_bytes[0..4] {
                [0xFF, 0xD8, 0xFF, _] => "image/jpeg",
                [0x89, 0x50, 0x4E, 0x47] => "image/png",
                [0x47, 0x49, 0x46, 0x38] => "image/gif",
                [0x52, 0x49, 0x46, 0x46] => "image/webp",
                _ => "image/jpeg", // Default fallback
            }
        } else {
            "image/jpeg"
        }
    };

    // Create the enhancement prompt
    let enhancement_prompt = format!(
        "Please enhance this image according to the following instructions: {}. \
        Focus on improving the overall quality while maintaining the original composition and style. \
        Keep the image in the same format as the input image. \
        Return the enhanced image as base64 encoded data source",
        params.enhancement_prompt
    );

    // Create content with image and text
    let contents = std::vec![Content {
        role: std::option::Option::Some(llm::vendors::gemini::role::Role::User),
        parts: std::vec![
            Part {
                text: std::option::Option::Some(enhancement_prompt),
                inline_data: std::option::Option::None,
                file_data: std::option::Option::None,
                function_response: std::option::Option::None,
                function_call: std::option::Option::None,
            },
            Part {
                text: std::option::Option::None,
                inline_data: std::option::Option::Some(InlineData {
                    mime_type: mime_type.to_string(),
                    data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_bytes),
                }),
                file_data: std::option::Option::None,
                function_response: std::option::Option::None,
                function_call: std::option::Option::None,
            },
        ],
    }];

    // Call Gemini API with Gemini 2.5 Flash Image Preview (latest image generation model)
    let response = match generate_gemini_conversation_response(
        contents,
        0.7, // Temperature for creative enhancement
        GeminiModel::Gemini25FlashImage, // Using Gemini 2.5 Flash Image Preview for image generation
        std::option::Option::None, // No tools needed
        std::option::Option::None, // No system instruction
    ).await {
        std::result::Result::Ok(output) => output,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Gemini API call failed: {e}"));
        }
    };

    // Extract the response data (text or image) and append MIME type
    let enhanced_image_data = match response {
        llm::vendors::gemini::gemini_output::GeminiOutput::Text(text) => {
            // If it's text, try to extract base64 data from it
            if text.contains("data:") {
                let parts: Vec<&str> = text.split(',').collect();
                if parts.len() == 2 {
                    // Already has MIME type prefix, use as is
                    text.clone()
                } else {
                    // No MIME type prefix, add it
                    format!("data:{};base64,{}", mime_type, text)
                }
            } else {
                // No MIME type prefix, add it
                format!("data:{};base64,{}", mime_type, text)
            }
        },
        llm::vendors::gemini::gemini_output::GeminiOutput::Image(inline_data) => {
            // If it's image data, append MIME type prefix
            format!("data:{};base64,{}", inline_data.mime_type, inline_data.data)
        },
        llm::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(_) => {
            return std::result::Result::Err("Unexpected function call response from Gemini".to_string());
        }
        llm::vendors::gemini::gemini_output::GeminiOutput::Mixed{text, ..} => {
            // Handle mixed response (text with function calls)
            if text.contains("data:") {
                let parts: Vec<&str> = text.split(',').collect();
                if parts.len() == 2 {
                    // Already has MIME type prefix, use as is
                    text.clone()
                } else {
                    // No MIME type prefix, add it
                    format!("data:{};base64,{}", mime_type, text)
                }
            } else {
                // No MIME type prefix, add it
                format!("data:{};base64,{}", mime_type, text)
            }
        },
    };


    // Create response data
    let response_data = serde_json::json!({
        "enhanced_image_data": enhanced_image_data,
        "original_prompt": params.enhancement_prompt,
        "enhancement_successful": true,
        "output_mime_type": mime_type
    });

    let full_response = agentloop::types::full_tool_response::FullToolResponse {
        tool_name: tool_name.clone(),
        response: response_data.clone(),
    };
    
    let user_response = agentloop::types::user_tool_response::UserToolResponse {
        tool_name,
        summary: format!(
            "Enhanced image using Quick Enhance Image tool with prompt: '{}'",
            params.enhancement_prompt
        ),
        data: std::option::Option::Some(response_data),
        icon: std::option::Option::Some("✨".to_string()),
    };
    
    // Consume credits before returning response
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::QUICK_ENHANCE_IMAGE;
    
    // Use provided credit_changes_params or construct from defaults
    let deduction_params = params.credit_changes_params.unwrap_or_else(|| {
        crate::queries::user_credit_allocation::CreditChangesParams {
            user_id,
            organization_id,
            credits_to_change: bigdecimal::BigDecimal::from(credits_to_consume),
            action_source: "agent_tool".to_string(),
            action_type: "quick_enhance_image".to_string(),
            entity_id: params.asset_id,
        }
    });
    
    if let Err(e) = crate::queries::user_credit_allocation::deduct_user_credits_with_transaction(pool, deduction_params).await {
        log::error!("Failed to deduct credits for user {} after quick enhance image: {}", user_id, e);
    }
    
    std::result::Result::Ok((full_response, user_response))
}
