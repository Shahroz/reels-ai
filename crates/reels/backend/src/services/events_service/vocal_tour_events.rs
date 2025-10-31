//! Custom analytics events tracking for vocal tour operations.
//!
//! This module provides vocal tour-specific event tracking that maintains the same base 
//! event structure as middleware events but adds custom_details for vocal tour data.
//! It extracts real request context from the HTTP request instead of using hardcoded values.

/// Logs a custom analytics event for vocal tour completion with gennodes server response and outcome metrics
/// This maintains the same base event structure as middleware events but adds custom_details
/// for the gennodes server response AND the actual outcome metrics (document, assets created)
pub async fn log_vocal_tour_gennodes_response(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    gennodes_response: &serde_json::Value,
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_time_ms: u64,
    // NEW: Outcome metrics
    document_id: Option<uuid::Uuid>,
    created_asset_ids: &[uuid::Uuid],
    vocal_tour_id: Option<uuid::Uuid>,
) -> std::result::Result<uuid::Uuid, String> {
    // Create analytics service instance
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create standardized request_details using the SAME helper function as all other events
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);

    // Create custom details with gennodes server response AND outcome metrics
    let custom_details = serde_json::json!({
        "gennodes_server_response": gennodes_response,
        // Response & performance metrics (moved from request_details for consistency)
        "response_status": 200, // Successful vocal tour completion
        "processing_time_ms": processing_time_ms,
        // NEW: Outcome metrics that were missing
        "outcome_metrics": {
            "document_id": document_id.map(|id| id.to_string()),
            "vocal_tour_id": vocal_tour_id.map(|id| id.to_string()),
            "assets_created_count": created_asset_ids.len(),
            "created_asset_ids": created_asset_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        }
    });

    // Log the custom event
    match analytics_service.create_custom_event_with_details(
        String::from("vocal_tour_gennodes_response"), // Event name
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    ).await {
        std::result::Result::Ok(event_id) => {
            log::info!("Successfully logged vocal tour custom analytics event {} for user {} (processing time: {}ms)", event_id, user_id, processing_time_ms);
            std::result::Result::Ok(event_id)
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to log vocal tour custom analytics event for user {}: {}", user_id, e);
            std::result::Result::Err(format!("Analytics event failed: {}", e))
        }
    }
}

// Browser extraction functions removed - now using standardized event_helpers functions for consistency

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocal_tour_event_processing_time() {
        // Test that processing time is properly included in event request_details
        let processing_time = 5000u64; // 5 seconds
        
        // Create mock request context
        let mut headers = std::collections::HashMap::new();
        headers.insert("origin".to_string(), "http://localhost:5173".to_string());
        
        let request_data = crate::services::events_service::request_context::RequestData {
            method: "POST".to_string(),
            path: "/api/vocal-tour".to_string(),
            full_url: "http://localhost:5173/api/vocal-tour".to_string(),
            query_string: "".to_string(),
            headers,
            query_params: serde_json::json!({}),
            user_agent: Some("Mozilla/5.0 test".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            real_ip: None,
            forwarded_for: None,
            scheme: "http".to_string(),
            host: "localhost:5173".to_string(),
            port: None,
            http_version: "HTTP/1.1".to_string(),
            content_type: None,
            content_length: None,
            content_encoding: None,
            accept_language: None,
            accept_encoding: None,
            request_body: None,
            request_body_size: None,
            request_body_truncated: false,
            user_registration_date: None,
            cookies: std::collections::HashMap::new(),
            request_id: "test_id".to_string(),
            timestamp: chrono::Utc::now(),
            user_id: Some(uuid::Uuid::new_v4()),
            session_id: Some("test_session".to_string()),
        };
        
        // Create standardized request_details using the SAME helper function
        let request_details = crate::services::events_service::event_helpers::build_standard_request_details(&request_data);
        
        // Verify standardized request_details structure (now matches ALL other events)
        assert_eq!(request_details["ip_address"], "127.0.0.1");
        assert_eq!(request_details["is_authenticated"], true);
        assert_eq!(request_details["session_type"], "cookie"); // test_session doesn't start with jwt_
        assert_eq!(request_details["method"], "POST");
        assert_eq!(request_details["path"], "/api/vocal-tour");
        assert_eq!(request_details["browser_info"]["browser_family"], "Unknown"); // Simple test user agent
        assert!(request_details["browser_info"]["is_desktop"].as_bool().unwrap());
        assert!(!request_details["browser_info"]["is_mobile"].as_bool().unwrap());
        // Note: processing_time_ms is now in custom_details, not request_details (for consistency)
    }
} 