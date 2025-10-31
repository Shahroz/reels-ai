//! Handler for quick image enhancement using various image enhancement models.
//!
//! Defines the `quick_enhance_image` HTTP handler under `/api/assets/quick-enhance-image`.
//! This handler takes an image (base64 encoded) and enhancement prompt, processes it through
//! various image enhancement models (currently Gemini 2.5 Flash Image), and returns the enhanced 
//! image data directly without creating asset records in the database.
//!
//! ## Process Flow
//!
//! 1. Validate image data format and decode base64
//! 2. Determine image MIME type from bytes
//! 3. Call image enhancement model with image and prompt
//! 4. Extract enhanced image data from response
//! 5. Return enhanced image data directly to client
//!
//! ## Security & Authorization
//!
//! - User must be authenticated (enforced by middleware)
//! - Image data is processed directly without storage
//! - Enhanced images are returned immediately without database persistence

use actix_web::{post, web, HttpResponse, Responder};
use crate::routes::assets::quick_enhance_image_request::QuickEnhanceImageRequest;
use crate::routes::assets::quick_enhance_image_response::QuickEnhanceImageResponse;
use crate::routes::assets::error_response::ErrorResponse;
use tracing::instrument;

/// Error type for Quick Enhance Image operations
#[derive(Debug, thiserror::Error)]
pub enum QuickEnhanceImageError {
    #[error("Invalid image data format: {0}")]
    InvalidImageData(String),
    #[error("Failed to decode base64 image data: {0}")]
    Base64DecodeError(String),
    #[error("Image enhancement API call failed: {0}")]
    EnhancementApiError(String),
    #[error("No enhanced image data received from enhancement service")]
    NoEnhancedImageData,
    #[error("Invalid enhancement prompt")]
    InvalidPrompt,
}

impl From<QuickEnhanceImageError> for HttpResponse {
    fn from(error: QuickEnhanceImageError) -> Self {
        let (status, error_message) = match error {
            QuickEnhanceImageError::InvalidImageData(_) => (actix_web::http::StatusCode::BAD_REQUEST, error.to_string()),
            QuickEnhanceImageError::Base64DecodeError(_) => (actix_web::http::StatusCode::BAD_REQUEST, error.to_string()),
            QuickEnhanceImageError::EnhancementApiError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            QuickEnhanceImageError::NoEnhancedImageData => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            QuickEnhanceImageError::InvalidPrompt => (actix_web::http::StatusCode::BAD_REQUEST, error.to_string()),
        };
        
        HttpResponse::build(status).json(ErrorResponse {
            error: error_message,
        })
    }
}

/// Validates and processes image data
fn validate_and_process_image_data(image_data: &str) -> Result<(Vec<u8>, String), QuickEnhanceImageError> {
    // Parse the base64 image data
    let image_data = if image_data.starts_with("data:") {
        // Extract base64 data from data URL
        let parts: Vec<&str> = image_data.split(',').collect();
        if parts.len() != 2 {
            return Err(QuickEnhanceImageError::InvalidImageData("Invalid data URL format".to_string()));
        }
        parts[1].to_string()
    } else {
        image_data.to_string()
    };

    // Decode base64 to get image bytes
    let image_bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &image_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(QuickEnhanceImageError::Base64DecodeError(e.to_string()));
        }
    };

    // Determine MIME type from image bytes or default to JPEG
    let mime_type = if image_bytes.len() >= 4 {
        match &image_bytes[0..4] {
            [0xFF, 0xD8, 0xFF, _] => "image/jpeg",
            [0x89, 0x50, 0x4E, 0x47] => "image/png",
            [0x47, 0x49, 0x46, 0x38] => "image/gif",
            [0x52, 0x49, 0x46, 0x46] => "image/webp",
            _ => "image/jpeg", // Default fallback
        }
    } else {
        "image/jpeg"
    };

    Ok((image_bytes, mime_type.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/assets/quick-enhance-image",
    tag = "Assets",
    request_body = QuickEnhanceImageRequest,
    responses(
        (status = 200, description = "Image enhanced successfully", body = QuickEnhanceImageResponse),
        (status = 400, description = "Bad Request - Invalid image data or prompt"),
        (status = 500, description = "Internal Server Error - Enhancement API Error")
    ),
    security(("user_auth" = []))
)]
#[post("/quick-enhance-image")]
#[instrument(skip(claims, req, http_req))]
pub async fn quick_enhance_image(
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<QuickEnhanceImageRequest>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let processing_start = std::time::Instant::now();
    let user_id = claims.user_id;
    let QuickEnhanceImageRequest {
        image_data,
        asset_id,
        enhancement_prompt,
        output_mime_type,
    } = req.into_inner();
    
    // Extract organization_id from header (if present)
    let organization_id = crate::services::credits_service::extract_organization_id_from_headers(&http_req);
    
    // Validate enhancement prompt
    if enhancement_prompt.trim().is_empty() {
        return HttpResponse::from(QuickEnhanceImageError::InvalidPrompt);
    }

    // Validate that we have either image_data or asset_id
    if image_data.is_none() && asset_id.is_none() {
        return HttpResponse::from(QuickEnhanceImageError::InvalidImageData("Either image or asset is required".to_string()));
    }

    if image_data.is_some() && asset_id.is_some() {
        return HttpResponse::from(QuickEnhanceImageError::InvalidImageData("Provide either image or asset, not both".to_string()));
    }

    // Create the enhancement prompt for Gemini
    let _gemini_prompt = format!(
        "Please enhance this image according to the following instructions: {}. \
        Return the enhanced image as base64 encoded data with the same format as the input image. \
        Focus on improving the overall quality while maintaining the original composition and style.",
        enhancement_prompt
    );

    // Prepare credit changes params for tool handler
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::QUICK_ENHANCE_IMAGE;
    let credit_changes_params = crate::queries::user_credit_allocation::CreditChangesParams {
        user_id,
        organization_id,
        credits_to_change: bigdecimal::BigDecimal::from(credits_to_consume),
        action_source: "api".to_string(),
        action_type: "quick_enhance_image".to_string(),
        entity_id: asset_id,
    };

    // Use the agent tool handler to process the image
    let params = crate::agent_tools::tool_params::quick_enhance_image_params::QuickEnhanceImageParams {
        image_data: image_data.clone(),
        asset_id: asset_id,
        enhancement_prompt: enhancement_prompt.clone(),
        output_mime_type: output_mime_type.clone(),
        user_id: Some(user_id),
        organization_id,
        credit_changes_params: Some(credit_changes_params),
    };

    let (full_response, _user_response) = match crate::agent_tools::handlers::handle_quick_enhance_image::handle_quick_enhance_image(params).await {
        Ok(responses) => responses,
        Err(e) => {
            log::error!("Quick Enhance Image failed for user {user_id}: {e}");
            return HttpResponse::from(QuickEnhanceImageError::EnhancementApiError(e));
        }
    };

    // Extract enhanced image data from response
    let enhanced_image_data = match full_response.response.get("enhanced_image_data") {
        Some(data) => match data.as_str() {
            Some(data_str) => data_str.to_string(),
            None => {
                log::error!("Enhanced image data is not a string for user {user_id}");
                return HttpResponse::from(QuickEnhanceImageError::NoEnhancedImageData);
            }
        },
        None => {
            log::error!("No enhanced image data in response for user {user_id}");
            return HttpResponse::from(QuickEnhanceImageError::NoEnhancedImageData);
        }
    };

    // Extract output MIME type from response
    let output_mime_type = match full_response.response.get("output_mime_type") {
        Some(mime_type) => match mime_type.as_str() {
            Some(mime_str) => mime_str.to_string(),
            None => {
                log::warn!("Output MIME type is not a string for user {user_id}, using default");
                "image/jpeg".to_string()
            }
        },
        None => {
            log::warn!("No output MIME type in response for user {user_id}, using default");
            "image/jpeg".to_string()
        }
    };

    let processing_time = processing_start.elapsed();
    
    let response = QuickEnhanceImageResponse {
        enhanced_image_data,
        original_prompt: enhancement_prompt,
        processing_time_ms: processing_time.as_millis() as u64,
        enhancement_successful: true,
        output_mime_type,
    };

    log::info!("Successfully enhanced image for user {} using Quick Enhance Image tool in {}ms", 
               user_id, processing_time.as_millis());

    HttpResponse::Ok().json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_and_process_image_data_valid_jpeg() {
        // Test with a minimal JPEG header
        let jpeg_data = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcUFhYaHSUfGhsjHBYWICwgIyYnKSopGR8tMC0oMCUoKSj/2wBDAQcHBwoIChMKChMoGhYaKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCj/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCdABmX/9k=";
        
        let result = validate_and_process_image_data(jpeg_data);
        assert!(result.is_ok());
        
        let (bytes, mime_type) = result.unwrap();
        assert_eq!(mime_type, "image/jpeg");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_validate_and_process_image_data_invalid_format() {
        let invalid_data = "invalid-data-format";
        
        let result = validate_and_process_image_data(invalid_data);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            QuickEnhanceImageError::Base64DecodeError(_) => {},
            _ => panic!("Expected Base64DecodeError"),
        }
    }

    #[test]
    fn test_validate_and_process_image_data_invalid_data_url() {
        let invalid_data_url = "data:invalid";
        
        let result = validate_and_process_image_data(invalid_data_url);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            QuickEnhanceImageError::InvalidImageData(_) => {},
            _ => panic!("Expected InvalidImageData"),
        }
    }
}
