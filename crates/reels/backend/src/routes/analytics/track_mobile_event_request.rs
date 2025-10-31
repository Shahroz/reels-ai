//! Request DTO for mobile analytics event tracking.
//!
//! This module defines the request structure for tracking analytics events
//! from mobile applications (iOS/Android). Events are stored in the same
//! analytics_events table used by Web, enabling cross-platform analysis.

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct TrackMobileEventRequest {
    /// Event name (e.g., "user_login_successful", "asset_upload_completed")
    #[schema(example = "user_login_successful")]
    pub event_name: String,
    
    /// Event-specific custom data stored in custom_details JSONB column
    #[serde(default)]
    #[schema(example = json!({"email_domain": "gmail.com"}))]
    pub custom_details: serde_json::Value,
    
    /// Device and platform information stored in request_details JSONB column
    /// Should include: platform, device_model, os_version, app_version
    #[serde(default)]
    #[schema(example = json!({
        "platform": "ios",
        "device_model": "iPhone15,3",
        "os_version": "17.2",
        "app_version": "1.0.0",
        "device_name": "iPhone 15 Pro"
    }))]
    pub device_info: serde_json::Value,
    
    /// Session identifier for tracking user sessions across events
    #[schema(example = "a1b2c3d4-e5f6-7890-abcd-ef1234567890")]
    pub session_id: Option<String>,
}

