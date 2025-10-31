// Add utoipa attribute macro
// use utoipa::path; // Commented out due to unused warning

// backend/src/routes/health.rs
use actix_web::{get, HttpResponse, Responder};

/// Health check endpoint handler.
/// Returns a simple OK response.
// Annotation for OpenAPI documentation
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health", // Group under 'management' tag in Swagger UI
    responses(
        (status = 200, description = "Health check successful")
    )
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}
