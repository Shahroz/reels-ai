//! Studio interaction events tracking for regenerate and tutorial workflows.
//!
//! This module provides custom event logging for Studio-related operations including
//! AI regeneration actions and tutorial interactions. These events enable analysis
//! of user engagement patterns and feature adoption in the Studio interface.

/// Log when user clicks the Regenerate button in Studio
#[cfg(feature = "events")]
pub async fn log_regenerate_clicked(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    asset_id: Option<uuid::Uuid>,
    request_context: &crate::services::events_service::request_context::RequestData,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Get user stats for context
    let user_stats = crate::services::events_service::event_helpers::get_user_count_stats(pool, user_id)
        .await
        .unwrap_or_else(|_| crate::services::events_service::event_helpers::UserCountStats {
            total_assets: 0,
            total_vocal_tours: 0,
            total_documents: 0,
            total_collections: 0,
        });
    
    // Custom details can be null for this event as requested
    let custom_details = if let Some(asset_id) = asset_id {
        serde_json::json!({
            "asset_id": asset_id.to_string(),
            "total_assets_for_user": user_stats.total_assets
        })
    } else {
        serde_json::Value::Null
    };
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
        "regenerate_clicked".to_string(),
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log regenerate clicked: {}", e))
}

/// Log when user clicks any button in tutorial (skip, back, next, finish)
#[cfg(feature = "events")]
pub async fn log_tutorial_button_clicked(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    button_name: &str,
    current_step: u32,
    step_name: &str,
    request_context: &crate::services::events_service::request_context::RequestData,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Create custom details with step and button information
    let custom_details = serde_json::json!({
        "button_name": button_name,
        "current_step": current_step,
        "step_name": step_name,
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
        "tutorial_button_clicked".to_string(),
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log tutorial button clicked: {}", e))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_completion_percentage_calculation() {
        // Test step 1 of 10
        let percentage = (1_f64 / 10.0 * 100.0).round();
        assert_eq!(percentage, 10.0);
        
        // Test step 5 of 10
        let percentage = (5_f64 / 10.0 * 100.0).round();
        assert_eq!(percentage, 50.0);
        
        // Test step 10 of 10
        let percentage = (10_f64 / 10.0 * 100.0).round();
        assert_eq!(percentage, 100.0);
    }
}
