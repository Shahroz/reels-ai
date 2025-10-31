//! Asset management events tracking for upload and enhancement workflows.
//!
//! This module provides custom event logging for asset-related operations including
//! file uploads, conversions, and AI-powered enhancements. These events enable analysis
//! of user content creation patterns and feature adoption rates.

/// Log when an asset upload is completed successfully
#[cfg(feature = "events")]
pub async fn log_asset_upload_completed(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    asset_id: uuid::Uuid,
    asset: &crate::db::assets::Asset,
    upload_duration_seconds: f64,
    conversion_applied: Option<&str>,
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_start: std::time::Instant,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Calculate processing time
    let processing_time_ms = processing_start.elapsed().as_millis() as u64;
    
    // Get user asset count
    let user_stats = crate::services::events_service::event_helpers::get_user_count_stats(pool, user_id)
        .await
        .unwrap_or_else(|_| crate::services::events_service::event_helpers::UserCountStats {
            total_assets: 1,
            total_vocal_tours: 0,
            total_documents: 0,
            total_collections: 0,
        });
    
    // Estimate file size (placeholder - could be enhanced with actual metadata)
    let file_size_mb = estimate_file_size_from_type(&asset.r#type);
    
    // Create custom details
    let custom_details = serde_json::json!({
        "asset_id": asset_id.to_string(),
        "file_type": asset.r#type,
        "file_size_mb": file_size_mb,
        "upload_duration_seconds": upload_duration_seconds,
        "conversion_applied": conversion_applied.unwrap_or("none"),
        "collection_id": asset.collection_id.map(|id| id.to_string()),
        "total_assets_for_user": user_stats.total_assets,
        "processing_time_ms": processing_time_ms
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
        "asset_upload_completed".to_string(),
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log asset upload completion: {}", e))
}

/// Log when asset enhancement is completed successfully
#[cfg(feature = "events")]
pub async fn log_asset_enhancement_completed(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    original_asset_ids: &[uuid::Uuid],
    enhanced_asset_ids: &[uuid::Uuid],
    retouch_prompt: &str,
    success_rate: f64,
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_start: std::time::Instant,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Calculate processing time
    let processing_time_ms = processing_start.elapsed().as_millis() as u64;
    let average_enhancement_time_ms = processing_time_ms / (original_asset_ids.len() as u64).max(1);
    
    // Estimate total file size for processed assets
    let total_file_size_mb = original_asset_ids.len() as f64 * 2.5; // Estimate 2.5MB per image
    
    // Create custom details
    let custom_details = serde_json::json!({
        "original_asset_ids": original_asset_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        "enhanced_asset_ids": enhanced_asset_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        "assets_processed": original_asset_ids.len(),
        "assets_created": enhanced_asset_ids.len(),
        "retouch_prompt": retouch_prompt,
        "prompt_length": retouch_prompt.len(),
        "processing_time_ms": processing_time_ms,
        "success_rate": success_rate,
        "total_file_size_mb": total_file_size_mb,
        "average_enhancement_time_ms": average_enhancement_time_ms
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
        "asset_enhancement_completed".to_string(),
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log asset enhancement completion: {}", e))
}

/// Estimate file size based on asset type (placeholder implementation)
fn estimate_file_size_from_type(asset_type: &str) -> f64 {
    match asset_type {
        t if t.starts_with("image/") => {
            if t.contains("heic") || t.contains("raw") || t.contains("dng") {
                8.0 // Larger for raw formats
            } else {
                2.5 // Average for compressed images
            }
        },
        t if t.starts_with("video/") => 15.0, // Average for videos
        _ => 1.0, // Documents and other files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_estimation() {
        assert_eq!(estimate_file_size_from_type("image/jpeg"), 2.5);
        assert_eq!(estimate_file_size_from_type("image/heic"), 8.0);
        assert_eq!(estimate_file_size_from_type("video/mp4"), 15.0);
        assert_eq!(estimate_file_size_from_type("application/pdf"), 1.0);
    }
} 
 
 