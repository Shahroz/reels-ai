//! Tracks analytics event for successful magic link login.
//!
//! Records the login event in the analytics_events table with request context.
//! This ensures magic link logins are tracked the same way as password logins.

#[cfg(feature = "events")]
/// Tracks a successful magic link login in analytics.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user who logged in
/// * `request_context` - Request context data for analytics
/// * `processing_start` - When request processing started
pub async fn track_magic_link_login_analytics(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    request_context: &crate::services::events_service::request_context::RequestData,
    processing_start: std::time::Instant,
) {
    match crate::services::events_service::auth_events::log_user_login_successful(
        pool,
        user_id,
        request_context,
        processing_start,
    ).await {
        std::result::Result::Ok(_) => {
            log::info!("Login analytics event tracked for user: {}", user_id);
        }
        std::result::Result::Err(e) => {
            log::warn!("Failed to track login analytics for user {}: {}", user_id, e);
        }
    }
}

#[cfg(not(feature = "events"))]
/// No-op when events feature is disabled.
pub async fn track_magic_link_login_analytics(
    _pool: &sqlx::PgPool,
    _user_id: uuid::Uuid,
    _request_context: &(),
    _processing_start: std::time::Instant,
) {
    // Analytics disabled
}

