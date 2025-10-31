//! Authentication events tracking for user registration and login flows.
//!
//! This module provides custom event logging for the complete authentication journey,
//! from registration attempts through successful logins. These events enable funnel
//! analysis of user onboarding and identify conversion bottlenecks in the auth flow.

/// Log when a user starts the registration process
#[cfg(feature = "events")]
pub async fn log_user_registration_started(
    pool: &sqlx::PgPool,
    email: &str,
    request_context: &crate::services::events_service::request_context::RequestData,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Extract email domain
    let email_domain = crate::services::events_service::event_helpers::extract_email_domain(email);
    
    // Create custom details - only email_domain as requested
    let custom_details = serde_json::json!({
        "email_domain": email_domain
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_anonymous(
        "user_registration_started".to_string(),
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log registration start: {}", e))
}

/// Log when user registration completes successfully
#[cfg(feature = "events")]
pub async fn log_user_registration_completed(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    email: &str,
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_start: std::time::Instant,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Calculate processing time
    let processing_time_ms = processing_start.elapsed().as_millis() as u64;
    
    // Extract email domain
    let email_domain = crate::services::events_service::event_helpers::extract_email_domain(email);
    
    // Create custom details - only email_domain and processing_time as requested
    let custom_details = serde_json::json!({
        "email_domain": email_domain,
        "processing_time_ms": processing_time_ms
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
        "user_registration_completed".to_string(),
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log registration completion: {}", e))
}

/// Log when user registration fails
#[cfg(feature = "events")]
pub async fn log_user_registration_failed(
    pool: &sqlx::PgPool,
    email: &str,
    failure_reason: &str,
    validation_errors: &[String],
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_start: std::time::Instant,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Calculate processing time
    let processing_time_ms = processing_start.elapsed().as_millis() as u64;
    
    // Extract email domain
    let email_domain = crate::services::events_service::event_helpers::extract_email_domain(email);
    
    // Create custom details
    let custom_details = serde_json::json!({
        "failure_reason": failure_reason,
        "email_domain": email_domain,
        "email": email,
        "validation_errors": validation_errors,
        "processing_time_ms": processing_time_ms
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_anonymous(
        "user_registration_failed".to_string(),
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log registration failure: {}", e))
}

/// Log when user login is successful
#[cfg(feature = "events")]
pub async fn log_user_login_successful(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_start: std::time::Instant,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.clone());
    
    // Create request details
    let request_details = crate::services::events_service::event_helpers::build_standard_request_details(request_context);
    
    // Calculate processing time
    let processing_time_ms = processing_start.elapsed().as_millis() as u64;
    
    // Create custom details - only processing time as requested
    let custom_details = serde_json::json!({
        "processing_time_ms": processing_time_ms
    });
    
    let event = crate::db::analytics_events::NewAnalyticsEvent::custom_authenticated(
        "user_login_successful".to_string(),
        user_id,
        request_details,
        custom_details,
        request_context.session_id.clone(),
    );
    
    analytics_service.create_event(event).await.map_err(|e| format!("Failed to log successful login: {}", e))
}

// Password strength evaluation removed - no longer needed for simplified events 
 
 