//! Tutorial event tracking endpoints for Studio.
//!
//! Provides HTTP handlers for logging tutorial interaction events including
//! skip, back, next, and finish actions. These events enable analysis of tutorial
//! effectiveness and user engagement patterns.

use actix_web::{post, web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::instrument;
use utoipa::ToSchema;

use crate::routes::error_response::ErrorResponse;
use crate::services::events_service::request_context::RequestData;
use crate::services::session_manager::HybridSessionManager;

/// Request body for tutorial event tracking
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TutorialEventRequest {
    /// Current step number (1-10)
    #[schema(example = 5, minimum = 1, maximum = 10)]
    pub current_step: u32,
    /// Name of the current step
    #[schema(example = "generate")]
    pub step_name: String,
    /// Name of the button clicked (skip, back, next, finish)
    #[schema(example = "next")]
    pub button_name: String,
}

/// Response for tutorial event tracking
#[derive(Debug, Serialize, ToSchema)]
pub struct TutorialEventResponse {
    /// Success message
    pub message: String,
    /// Event ID for tracking
    pub event_id: Option<String>,
}

/// Log tutorial button click event
#[utoipa::path(
    post,
    path = "/api/studio/tutorial/button-click",
    request_body = TutorialEventRequest,
    responses(
        (status = 200, description = "Tutorial button event logged successfully", body = TutorialEventResponse),
        (status = 400, description = "Bad Request - Invalid step data or button name"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Studio Tutorial Events"
)]
#[post("/tutorial/button-click")]
#[instrument(skip(pool, claims, req, http_req, session_manager))]
pub async fn log_tutorial_button_click(
    pool: web::Data<PgPool>,
    session_manager: web::Data<Arc<HybridSessionManager>>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    http_req: HttpRequest,
    req: web::Json<TutorialEventRequest>,
) -> impl Responder {
    let request_data = req.into_inner();
    let user_id = claims.user_id;
    
    // Validate step range
    if request_data.current_step < 1 || request_data.current_step > 10 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid step: must be between 1 and 10".to_string(),
        });
    }
    
    // Validate button name
    let valid_buttons = ["skip", "back", "next", "finish"];
    if !valid_buttons.contains(&request_data.button_name.as_str()) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid button name: must be one of skip, back, next, finish".to_string(),
        });
    }
    
    // Get session ID from request
    let session_id = match session_manager.get_or_create_session(user_id).await {
        Ok(session_id) => Some(session_id),
        Err(e) => {
            log::warn!("Failed to get session for tutorial event: {}", e);
            None
        }
    };
    
    // Build request context
    let request_context = RequestData {
        method: http_req.method().to_string(),
        path: http_req.path().to_string(),
        full_url: format!("{}://{}{}", 
            http_req.connection_info().scheme(),
            http_req.connection_info().host(),
            http_req.path()
        ),
        query_string: http_req.query_string().to_string(),
        headers: http_req.headers().iter()
            .filter_map(|(name, value)| {
                value.to_str().ok().map(|v| (name.to_string(), v.to_string()))
            })
            .collect(),
        query_params: serde_json::Value::Null,
        user_agent: http_req.headers().get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        ip_address: http_req.connection_info().realip_remote_addr().map(|s| s.to_string()),
        real_ip: http_req.connection_info().realip_remote_addr().map(|s| s.to_string()),
        forwarded_for: http_req.headers().get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        scheme: http_req.connection_info().scheme().to_string(),
        host: http_req.connection_info().host().to_string(),
        port: None,
        http_version: format!("{:?}", http_req.version()),
        content_type: http_req.headers().get("content-type")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        content_length: None,
        content_encoding: http_req.headers().get("content-encoding")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        accept_language: http_req.headers().get("accept-language")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        accept_encoding: http_req.headers().get("accept-encoding")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        request_body: None,
        request_body_size: None,
        request_body_truncated: false,
        user_registration_date: None,
        cookies: std::collections::HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        user_id: Some(user_id),
        session_id,
    };
    
    #[cfg(feature = "events")]
    match crate::services::events_service::studio_events::log_tutorial_button_clicked(
        &pool,
        user_id,
        &request_data.button_name,
        request_data.current_step,
        &request_data.step_name,
        &request_context,
    ).await {
        Ok(event_id) => HttpResponse::Ok().json(TutorialEventResponse {
            message: format!("Tutorial {} event logged successfully", request_data.button_name),
            event_id: Some(event_id.to_string()),
        }),
        Err(e) => {
            log::error!("Failed to log tutorial {} event: {}", request_data.button_name, e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to log tutorial event".to_string(),
            })
        }
    }
    
    #[cfg(not(feature = "events"))]
    HttpResponse::Ok().json(TutorialEventResponse {
        message: format!("Tutorial {} event logged successfully (events disabled)", request_data.button_name),
        event_id: None,
    })
}