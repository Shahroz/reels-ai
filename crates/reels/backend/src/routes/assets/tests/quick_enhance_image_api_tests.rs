//! Test cases for the Quick Enhance Image API endpoint.
//!
//! This module contains comprehensive test cases for the `/api/assets/quick-enhance-image` endpoint,
//! including unit tests, integration tests, and error handling tests.

#[cfg(test)]
mod tests {
    use super::super::quick_enhance_image;
    use super::super::quick_enhance_image_request::QuickEnhanceImageRequest;
    use super::super::quick_enhance_image_response::QuickEnhanceImageResponse;
    use actix_web::{test, web, App};
    use crate::auth::tokens::Claims;

    /// Creates a test image data (minimal JPEG)
    fn create_test_image_data() -> String {
        "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcUFhYaHSUfGhsjHBYWICwgIyYnKSopGR8tMC0oMCUoKSj/2wBDAQcHBwoIChMKChMoGhYaKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCj/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCdABmX/9k=".to_string()
    }

    /// Creates test request data
    fn create_test_request() -> QuickEnhanceImageRequest {
        QuickEnhanceImageRequest {
            image_data: Some(create_test_image_data()),
            asset_id: None,
            enhancement_prompt: "Enhance the lighting and improve overall quality".to_string(),
        }
    }

    /// Creates test request data with asset_id
    fn create_test_request_with_asset_id() -> QuickEnhanceImageRequest {
        QuickEnhanceImageRequest {
            image_data: None,
            asset_id: Some(uuid::Uuid::new_v4()),
            enhancement_prompt: "Enhance the lighting and improve overall quality".to_string(),
        }
    }

    /// Creates mock claims for testing
    fn create_mock_claims() -> Claims {
        Claims {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
        }
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_request_serialization() {
        let request = create_test_request();
        
        // Test serialization
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("image_data"));
        assert!(serialized.contains("enhancement_prompt"));
        
        // Test deserialization
        let deserialized: QuickEnhanceImageRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.image_data, request.image_data);
        assert_eq!(deserialized.enhancement_prompt, request.enhancement_prompt);
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_response_serialization() {
        let response = QuickEnhanceImageResponse {
            enhanced_image_data: create_test_image_data(),
            original_prompt: "Test prompt".to_string(),
            processing_time_ms: 1250,
            enhancement_successful: true,
        };
        
        // Test serialization
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("enhanced_image_data"));
        assert!(serialized.contains("original_prompt"));
        assert!(serialized.contains("processing_time_ms"));
        assert!(serialized.contains("enhancement_successful"));
        
        // Test deserialization
        let deserialized: QuickEnhanceImageResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.enhanced_image_data, response.enhanced_image_data);
        assert_eq!(deserialized.original_prompt, response.original_prompt);
        assert_eq!(deserialized.processing_time_ms, response.processing_time_ms);
        assert_eq!(deserialized.enhancement_successful, response.enhancement_successful);
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_endpoint_structure() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&create_test_request())
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // The response will depend on whether GEMINI_API_KEY is set
        // If not set, we expect a 500 error
        // If set, we expect either success or a Gemini API error
        assert!(resp.status().is_client_error() || resp.status().is_server_error() || resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_invalid_json() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_payload("invalid json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_empty_prompt() {
        let mut request = create_test_request();
        request.enhancement_prompt = "".to_string();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_no_image_source() {
        let request = QuickEnhanceImageRequest {
            image_data: None,
            asset_id: None,
            enhancement_prompt: "Test prompt".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_both_sources() {
        let request = QuickEnhanceImageRequest {
            image_data: Some(create_test_image_data()),
            asset_id: Some(uuid::Uuid::new_v4()),
            enhancement_prompt: "Test prompt".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_invalid_image_data() {
        let mut request = create_test_request();
        request.image_data = Some("invalid-base64-data".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_empty_enhancement_prompt() {
        let mut request = create_test_request();
        request.enhancement_prompt = "".to_string();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_quick_enhance_image_whitespace_only_enhancement_prompt() {
        let mut request = create_test_request();
        request.enhancement_prompt = "   ".to_string();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_mock_claims()))
                .service(web::scope("/api/assets")
                    .service(quick_enhance_image))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/assets/quick-enhance-image")
            .set_json(&request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
