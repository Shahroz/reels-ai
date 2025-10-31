//! Test module for custom analytics events functionality.
//!
//! This module contains integration tests for custom events that verify:
//! - Custom events can be created with both details and custom_details
//! - The data structure maintains consistency between middleware and custom events
//! - Custom details are properly stored and retrievable
//! - Validation works correctly for custom event data

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_custom_event_creation_with_gennodes_response() {
        // This test verifies that we can create a custom event with gennodes server response
        // similar to what would happen in vocal tour processing
        
        // Mock analytics service
        let pool = create_test_pool().await;
        let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
        
        // Test user
        let user_id = uuid::Uuid::new_v4();
        
        // Standard event details (same structure as middleware events)
        let details = serde_json::json!({
            "ip_address": "127.0.0.1",
            "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "browser_info": {
                "browser_family": "Chrome",
                "browser_version": "137.0.0.0",
                "os_family": "macOS",
                "is_mobile": false,
                "is_desktop": true
            },
            "origin": "http://localhost:5173",
            "referer": "http://localhost:5173/",
            "is_authenticated": true,
            "session_type": "cookie",
                         "response_status": 200,
             "processing_time_ms": 176340
        });
        
        // Custom event details with gennodes server response
        let custom_details = serde_json::json!({
            "gennodes_server_response": {
                "status": "success",
                "data": {
                    "PropertyDescription": {
                        "title": "Beautiful 3BR Home",
                        "body": "<p>Stunning property with...</p>",
                        "voiceOverTranscript": "Welcome to this beautiful home..."
                    }
                },
                "processing_time": 15000,
                "images_generated": 12
            }
        });
        
        // Create the custom event
        let result = analytics_service.create_custom_event_with_details(
            String::from("vocal_tour_gennodes_response"),
            user_id,
            details,
            custom_details,
            Some(String::from("ses_test_123")),
        ).await;
        
        // Verify event was created successfully
        assert!(result.is_ok(), "Custom event creation should succeed");
        let event_id = result.unwrap();
        
        // Verify the event can be retrieved and has correct structure
        let retrieved_event = crate::queries::analytics::insert_analytics_event::get_analytics_event_by_id(&pool, event_id).await;
        assert!(retrieved_event.is_ok(), "Should be able to retrieve the created event");
        
        let event = retrieved_event.unwrap().unwrap();
        assert_eq!(event.event_name, "vocal_tour_gennodes_response");
        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.source.as_str(), "custom");
        
        // Verify details structure matches middleware events
        assert_eq!(event.details["ip_address"], "127.0.0.1");
        assert_eq!(event.details["browser_info"]["browser_family"], "Chrome");
        assert_eq!(event.details["is_authenticated"], true);
        
        // Verify custom_details contains gennodes response
        assert!(event.custom_details.get("gennodes_server_response").is_some());
        assert_eq!(event.custom_details["gennodes_server_response"]["status"], "success");
        assert_eq!(event.custom_details["gennodes_server_response"]["images_generated"], 12);
        
        println!("✅ Custom event test passed - event created with ID: {}", event_id);
    }
    
    #[tokio::test]
    async fn test_middleware_vs_custom_event_consistency() {
        // This test verifies that middleware and custom events have consistent base structure
        // for analytics segmentation capabilities
        
        let pool = create_test_pool().await;
        let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
        
        let user_id = uuid::Uuid::new_v4();
        
        // Base details structure (same for both)
        let base_details = serde_json::json!({
            "ip_address": "127.0.0.1",
            "user_agent": "test-agent",
            "browser_info": {
                "browser_family": "Chrome",
                "os_family": "macOS",
                "is_mobile": false
            },
            "is_authenticated": true,
            "response_status": 200
        });
        
        // Create middleware event (via helper method)
        let middleware_event = crate::db::analytics_events::NewAnalyticsEvent::middleware_authenticated(
            String::from("GET /api/test"),
            user_id,
            base_details.clone(),
            Some(String::from("session_123")),
        );
        
        // Create custom event (via helper method)
        let custom_event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
            String::from("vocal_tour_completed"),
            user_id,
            base_details.clone(),
            serde_json::json!({"gennodes_server_response": {"status": "success"}}),
            Some(String::from("session_123")),
        );
        
        // Verify both events have consistent base structure
        assert_eq!(middleware_event.user_id, custom_event.user_id);
        assert_eq!(middleware_event.session_id, custom_event.session_id);
        assert_eq!(middleware_event.details["ip_address"], custom_event.details["ip_address"]);
        assert_eq!(middleware_event.details["browser_info"], custom_event.details["browser_info"]);
        
        // Verify source differences
        assert_eq!(middleware_event.source.as_str(), "middleware");
        assert_eq!(custom_event.source.as_str(), "custom");
        
        // Verify custom_details differences
        assert_eq!(middleware_event.custom_details, serde_json::json!({}));
        assert_ne!(custom_event.custom_details, serde_json::json!({}));
        assert!(custom_event.custom_details.get("gennodes_server_response").is_some());
        
        println!("✅ Event consistency test passed");
    }
    
    async fn create_test_pool() -> sqlx::PgPool {
        // Load environment variables
        dotenvy::dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for integration tests");
            
        sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }
} 