//! Health check endpoint for the backend API.
//!
//! This endpoint is used to verify that the server is running and responsive.
//! It returns a simple 200 OK response with a JSON status message.

use actix_web::{HttpResponse, Responder};

/// Health check endpoint handler.
/// 
/// Returns a 200 OK response indicating the server is healthy.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = String)
    ),
    tag = "Health"
)]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "message": "Server is running"
    }))
}

