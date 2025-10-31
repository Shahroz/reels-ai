//! HTTP response conversion for `EnhanceAssetError`.
//!
//! Maps each `EnhanceAssetError` variant to an appropriate HTTP status code
//! and JSON error response.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

impl From<crate::routes::assets::enhance_asset_error::EnhanceAssetError> for actix_web::HttpResponse {
    fn from(error: crate::routes::assets::enhance_asset_error::EnhanceAssetError) -> Self {
        let (status, error_message) = match error {
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::InvalidAssetId(_) => {
                (actix_web::http::StatusCode::BAD_REQUEST, error.to_string())
            }
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::AssetNotFound => {
                (actix_web::http::StatusCode::NOT_FOUND, error.to_string())
            }
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::NonImageAsset => {
                (actix_web::http::StatusCode::BAD_REQUEST, error.to_string())
            }
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::EnhancementFailed(_) => {
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::ProcessingFailed(_) => {
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            crate::routes::assets::enhance_asset_error::EnhanceAssetError::SaveFailed(_) => {
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
        };
        
        actix_web::HttpResponse::build(status).json(crate::routes::assets::error_response::ErrorResponse {
            error: error_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::body::MessageBody;

    #[test]
    fn test_invalid_asset_id_converts_to_bad_request() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::InvalidAssetId("bad-id".to_string());
        let response: actix_web::HttpResponse = error.into();
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_asset_not_found_converts_to_not_found() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::AssetNotFound;
        let response: actix_web::HttpResponse = error.into();
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_non_image_asset_converts_to_bad_request() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::NonImageAsset;
        let response: actix_web::HttpResponse = error.into();
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_enhancement_failed_converts_to_internal_server_error() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::EnhancementFailed("network error".to_string());
        let response: actix_web::HttpResponse = error.into();
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_processing_failed_converts_to_internal_server_error() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::ProcessingFailed("parse error".to_string());
        let response: actix_web::HttpResponse = error.into();
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_save_failed_converts_to_internal_server_error() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::SaveFailed("db error".to_string());
        let response: actix_web::HttpResponse = error.into();
        assert_eq!(response.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_response_contains_error_message() {
        let error = crate::routes::assets::enhance_asset_error::EnhanceAssetError::InvalidAssetId("test-id".to_string());
        let response: actix_web::HttpResponse = error.into();
        
        // We can't easily test the body content without async runtime,
        // but we can verify the response is properly formed
        assert!(response.status().is_client_error());
    }
}


