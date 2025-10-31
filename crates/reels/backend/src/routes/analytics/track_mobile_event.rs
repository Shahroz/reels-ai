//! Handler for tracking analytics events from mobile applications.
//!
//! This endpoint receives events from iOS/Android apps and stores them in the
//! same analytics_events table used by Web, enabling cross-platform analysis.
//! Events are differentiated by platform field in request_details JSONB column.

#[utoipa::path(
    post,
    path = "/api/analytics/track-mobile-event",
    tag = "Analytics",
    request_body = crate::routes::analytics::track_mobile_event_request::TrackMobileEventRequest,
    responses(
        (status = 200, description = "Event tracked successfully", body = crate::db::analytics_events::AnalyticsEvent),
        (status = 401, description = "Unauthorized - valid JWT token required"),
        (status = 500, description = "Internal server error - failed to track event"),
    ),
    security(("bearer_auth" = []))
)]
#[actix_web::post("/track-mobile-event")]
#[tracing::instrument(skip(pool, http_req, body))]
pub async fn track_mobile_event(
    pool: actix_web::web::Data<sqlx::PgPool>,
    http_req: actix_web::HttpRequest,
    body: actix_web::web::Json<crate::routes::analytics::track_mobile_event_request::TrackMobileEventRequest>,
) -> actix_web::Result<impl actix_web::Responder> {
    // Extract user_id from AuthenticatedUser enum
    let user_id = if let Some(auth_user) = actix_web::HttpMessage::extensions(&http_req).get::<crate::middleware::auth::AuthenticatedUser>() {
        match auth_user {
            crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
            crate::middleware::auth::AuthenticatedUser::ApiKey(id) => *id,
        }
    } else {
        return Ok(actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required"
        })));
    };
    
    // Validate event_name length (prevent DoS via huge event names)
    if body.event_name.len() > 255 {
        return Ok(actix_web::HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid request",
            "message": "event_name too long (max 255 characters)"
        })));
    }
    
    // Validate JSON depth to prevent stack overflow attacks
    fn check_json_depth(value: &serde_json::Value, current: usize, max: usize) -> bool {
        if current > max {
            return false;
        }
        match value {
            serde_json::Value::Object(map) => {
                map.values().all(|v| check_json_depth(v, current + 1, max))
            }
            serde_json::Value::Array(arr) => {
                arr.iter().all(|v| check_json_depth(v, current + 1, max))
            }
            _ => true,
        }
    }
    
    if !check_json_depth(&body.custom_details, 0, 10) || !check_json_depth(&body.device_info, 0, 10) {
        return Ok(actix_web::HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid request",
            "message": "JSON nesting too deep (max 10 levels)"
        })));
    }
    
    // Create analytics service using existing infrastructure
    let analytics_service = crate::services::analytics::analytics_event_service::AnalyticsEventService::new(pool.get_ref().clone());
    
    // Build request_details with platform information
    // This differentiates mobile from web events in analytics queries
    let request_details = serde_json::json!({
        "platform": body.device_info.get("platform").and_then(|v| v.as_str()).unwrap_or("ios"),
        "device_info": body.device_info,
    });
    
    // Create event and return atomically (prevents race conditions)
    // Uses INSERT...RETURNING to get complete event in single DB operation
    let event = analytics_service
        .create_custom_event_with_details_returning(
            body.event_name.clone(),
            user_id,
            request_details,
            body.custom_details.clone(),
            body.session_id.clone(),
        )
        .await
        .map_err(|e| {
            log::error!("Failed to create mobile analytics event: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to track event")
        })?;
    
    log::info!(
        "Mobile event tracked: {} for user {} (event_id: {})",
        body.event_name,
        user_id,
        event.id
    );
    
    Ok(actix_web::HttpResponse::Ok().json(event))
}

