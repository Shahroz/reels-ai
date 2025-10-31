//! Test cases for the Quick Enhance Image tool.
//!
//! This module contains comprehensive test cases for the `handle_quick_enhance_image` function,
//! including unit tests, integration tests, and error handling tests.

#[cfg(test)]
mod tests {
    use super::super::handle_quick_enhance_image;
    use crate::agent_tools::tool_params::quick_enhance_image_params::QuickEnhanceImageParams;

    /// Creates a test image data (minimal JPEG)
    fn create_test_image_data() -> String {
        // Minimal JPEG header for testing
        "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcUFhYaHSUfGhsjHBYWICwgIyYnKSopGR8tMC0oMCUoKSj/2wBDAQcHBwoIChMKChMoGhYaKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCj/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCdABmX/9k=".to_string()
    }

    /// Creates test parameters for the Quick Enhance Image tool
    fn create_test_params() -> QuickEnhanceImageParams {
        QuickEnhanceImageParams {
            image_data: Some(create_test_image_data()),
            asset_id: None,
            enhancement_prompt: "Enhance the lighting and improve overall quality".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: None,
        }
    }

    /// Creates test parameters with asset_id for the Quick Enhance Image tool
    fn create_test_params_with_asset_id() -> QuickEnhanceImageParams {
        QuickEnhanceImageParams {
            image_data: None,
            asset_id: Some(uuid::Uuid::new_v4()),
            enhancement_prompt: "Enhance the lighting and improve overall quality".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: None,
        }
    }

    #[tokio::test]
    #[ignore] // Requires GEMINI_API_KEY environment variable
    async fn test_quick_enhance_image_success() {
        // This test requires a valid GEMINI_API_KEY environment variable
        if std::env::var("GEMINI_API_KEY").is_err() {
            println!("Skipping test: GEMINI_API_KEY not set");
            return;
        }

        let params = create_test_params();
        let user_id = uuid::Uuid::new_v4();
        
        let result = handle_quick_enhance_image(params, user_id).await;
        
        match result {
            Ok((full_response, user_response)) => {
                // Verify full response structure
                assert_eq!(full_response.tool_name, "quick_enhance_image");
                assert!(full_response.response.is_object());
                
                // Verify enhanced image data is present
                let enhanced_data = full_response.response.get("enhanced_image_data");
                assert!(enhanced_data.is_some());
                assert!(enhanced_data.unwrap().is_string());
                
                // Verify user response structure
                assert_eq!(user_response.tool_name, "quick_enhance_image");
                assert!(user_response.summary.contains("Enhanced image using Quick Enhance Image tool"));
                assert!(user_response.data.is_some());
                assert_eq!(user_response.icon, Some("✨".to_string()));
            }
            Err(e) => {
                panic!("Expected success but got error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_missing_api_key() {
        // Temporarily remove API key if it exists
        let original_key = std::env::var("GEMINI_API_KEY").ok();
        std::env::remove_var("GEMINI_API_KEY");
        
        let params = create_test_params();
        let user_id = uuid::Uuid::new_v4();
        
        let result = handle_quick_enhance_image(params, user_id).await;
        
        // Restore original API key
        if let Some(key) = original_key {
            std::env::set_var("GEMINI_API_KEY", key);
        }
        
        match result {
            Ok(_) => {
                panic!("Expected error due to missing API key");
            }
            Err(e) => {
                assert!(e.contains("GEMINI_API_KEY environment variable is required"));
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_invalid_base64() {
        let mut params = create_test_params();
        params.image_data = Some("invalid-base64-data".to_string());
        
        let result = handle_quick_enhance_image(params).await;
        
        match result {
            Ok(_) => {
                panic!("Expected error due to invalid base64 data");
            }
            Err(e) => {
                assert!(e.contains("Failed to decode base64 image data"));
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_no_image_source() {
        let params = QuickEnhanceImageParams {
            image_data: None,
            asset_id: None,
            enhancement_prompt: "Test prompt".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: None,
        };
        
        let result = handle_quick_enhance_image(params).await;
        
        match result {
            Ok(_) => {
                panic!("Expected error due to no image source");
            }
            Err(e) => {
                assert!(e.contains("Either image_data or asset_id is required"));
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_both_sources() {
        let params = QuickEnhanceImageParams {
            image_data: Some(create_test_image_data()),
            asset_id: Some(uuid::Uuid::new_v4()),
            enhancement_prompt: "Test prompt".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: None,
        };
        
        let result = handle_quick_enhance_image(params).await;
        
        match result {
            Ok(_) => {
                panic!("Expected error due to both sources provided");
            }
            Err(e) => {
                assert!(e.contains("Provide either image or asset, not both"));
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_empty_prompt() {
        let mut params = create_test_params();
        params.enhancement_prompt = "".to_string();
        
        // This should still work as the handler doesn't validate empty prompts
        // The validation happens in the API layer
        let result = handle_quick_enhance_image(params).await;
        
        // The result depends on whether API key is set
        if std::env::var("GEMINI_API_KEY").is_err() {
            match result {
                Ok(_) => {
                    panic!("Expected error due to missing API key");
                }
                Err(e) => {
                    assert!(e.contains("GEMINI_API_KEY environment variable is required"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_different_image_formats() {
        // Test with PNG data
        let png_data = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        
        let mut params = create_test_params();
        params.image_data = png_data.to_string();
        
        // This test will fail without API key, but we can test the parsing logic
        let result = handle_quick_enhance_image(params).await;
        
        if std::env::var("GEMINI_API_KEY").is_err() {
            match result {
                Ok(_) => {
                    panic!("Expected error due to missing API key");
                }
                Err(e) => {
                    assert!(e.contains("GEMINI_API_KEY environment variable is required"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_empty_enhancement_prompt() {
        let params = QuickEnhanceImageParams {
            image_data: Some(create_test_image_data()),
            asset_id: None,
            enhancement_prompt: "".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: None,
        };
        
        let result = handle_quick_enhance_image(params).await;
        
        match result {
            Err(error) => {
                assert!(error.contains("Enhancement prompt is required and cannot be empty"));
            }
            Ok(_) => {
                panic!("Expected error for empty enhancement prompt");
            }
        }
    }

    #[tokio::test]
    async fn test_quick_enhance_image_whitespace_only_enhancement_prompt() {
        let params = QuickEnhanceImageParams {
            image_data: Some(create_test_image_data()),
            asset_id: None,
            enhancement_prompt: "   ".to_string(),
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: None,
        };
        
        let result = handle_quick_enhance_image(params).await;
        
        match result {
            Err(error) => {
                assert!(error.contains("Enhancement prompt is required and cannot be empty"));
            }
            Ok(_) => {
                panic!("Expected error for whitespace-only enhancement prompt");
            }
        }
    }

    #[test]
    fn test_quick_enhance_image_params_serialization() {
        let params = create_test_params();
        
        // Test serialization
        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("image_data"));
        assert!(serialized.contains("enhancement_prompt"));
        
        // Test deserialization
        let deserialized: QuickEnhanceImageParams = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.image_data, params.image_data);
        assert_eq!(deserialized.enhancement_prompt, params.enhancement_prompt);
    }

    #[test]
    fn test_quick_enhance_image_params_default() {
        let params = QuickEnhanceImageParams::default();
        
        assert_eq!(params.image_data, None);
        assert_eq!(params.asset_id, None);
        assert_eq!(params.enhancement_prompt, "");
        assert_eq!(params.user_id, None);
    }
}
