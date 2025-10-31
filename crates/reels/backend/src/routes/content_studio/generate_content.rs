//! Handler for generating new content using LLM.

use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use validator::Validate;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::services::content_generation_functions::generate_new_content;
use super::requests::GenerateContentRequest;
use super::responses::ContentGenerationResponse;

#[utoipa::path(
    post,
    path = "/api/content-studio/generate",
    tag = "Content Studio",
    request_body = GenerateContentRequest,
    responses(
        (status = 201, description = "Content generated successfully", body = ContentGenerationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[post("/generate")]
#[instrument(skip(pool, claims))]
pub async fn generate_content(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<GenerateContentRequest>,
) -> impl Responder {
    log::info!("Content Studio generate_content handler called for user: {}", claims.user_id);
    log::info!("Request payload: title='{}', prompt length={}", 
               req.title, req.prompt.len());

    // Validate the request body
    if let Err(validation_errors) = req.validate() {
        tracing::warn!(
            "Validation failed for generate content request by user {}: {}",
            claims.user_id,
            validation_errors
        );
        return HttpResponse::UnprocessableEntity().json(ErrorResponse {
            error: format!("Validation failed: {validation_errors}"),
        });
    }

    let user_id = claims.user_id;
    let generation_start = std::time::Instant::now();

    // Generate new content using the standalone function
    match generate_new_content(
        pool.get_ref(),
        &req.prompt,
        user_id,
        req.title.clone(),
        req.collection_id,
    ).await {
        Ok(generated_document) => {
            let generation_time_ms = generation_start.elapsed().as_millis() as u64;
            
            tracing::info!(
                "Content generation completed for user {} with title '{}' in {}ms",
                user_id,
                req.title,
                generation_time_ms
            );

            let response = ContentGenerationResponse {
                document: generated_document,
                source_document_id: None, // No source document for new generation
                model_used: "gemini-2.5-pro".to_string(),
                generation_time_ms,
                has_provenance: false,
            };

            HttpResponse::Created().json(response)
        }
        Err(e) => {
            tracing::error!(
                "Content generation failed for user {} with title '{}': {}",
                user_id,
                req.title,
                e
            );

            // Check for specific error types
            if e.contains("LLM") || e.contains("generation") {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Content generation service temporarily unavailable".to_string(),
                })
            } else if e.contains("rate limit") {
                HttpResponse::TooManyRequests().json(ErrorResponse {
                    error: "Rate limit exceeded. Please try again later.".to_string(),
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to generate content".to_string(),
                })
            }
        }
    }
}