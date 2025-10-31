//! Handler for transforming existing documents using LLM.

use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use validator::Validate;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::services::content_generation_functions::transform_document_content;
use crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user;
use super::requests::TransformDocumentRequest;
use super::responses::ContentGenerationResponse;

#[utoipa::path(
    post,
    path = "/api/content-studio/transform",
    tag = "Content Studio",
    request_body = TransformDocumentRequest,
    responses(
        (status = 200, description = "Document transformed successfully", body = ContentGenerationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Document access denied"),
        (status = 404, description = "Source document not found"),
        (status = 422, description = "Validation Error", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[post("/transform")]
#[instrument(skip(pool, claims))]
pub async fn transform_document(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<TransformDocumentRequest>,
) -> impl Responder {
    log::info!("Content Studio transform_document handler called for user: {}", claims.user_id);
    log::info!("Request payload: source_document_id={}", req.source_document_id);
    
    // Validate the request body
    if let Err(validation_errors) = req.validate() {
        tracing::warn!(
            "Validation failed for transform document request by user {}: {}",
            claims.user_id,
            validation_errors
        );
        return HttpResponse::UnprocessableEntity().json(ErrorResponse {
            error: format!("Validation failed: {validation_errors}"),
        });
    }

    let user_id = claims.user_id;
    
    // Find the source document
    let source_document = match find_document_by_id_and_user(
        pool.get_ref(),
        req.source_document_id,
        user_id,
    ).await {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            tracing::warn!(
                "Source document {} not found or access denied for user {}",
                req.source_document_id,
                user_id
            );
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Source document not found or access denied".to_string(),
            });
        }
        Err(e) => {
            tracing::error!(
                "Database error when finding source document {} for user {}: {}",
                req.source_document_id,
                user_id,
                e
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database error while accessing source document".to_string(),
            });
        }
    };

    let generation_start = std::time::Instant::now();

    // Determine transformation prompt - either from request or generated from template
    let final_transformation_prompt = match (&req.transformation_prompt, &req.template_document_id) {
        (Some(prompt), None) => {
            // Use provided transformation prompt
            prompt.clone()
        }
        (None, Some(template_id)) => {
            // Fetch template document and generate transformation prompt
            match find_document_by_id_and_user(
                pool.get_ref(),
                *template_id,
                user_id,
            ).await {
                std::result::Result::Ok(Some(template_doc)) => {
                    // Verify this is actually a template document
                    if !template_doc.sources.contains(&"content_studio_template".to_string()) {
                        tracing::warn!(
                            "Document {} is not a template document for user {}",
                            template_id,
                            user_id
                        );
                        return actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
                            error: "Specified document is not a template".to_string(),
                        });
                    }

                    // Generate template-based transformation prompt
                    format!(
                        "Please transform the source document into the template document format. \
                         Use the data and content from the source document, but retain the format, \
                         structure, and style of this template:\n\n---TEMPLATE---\n{}\n\n---END TEMPLATE---\n\n\
                         Transform the source content to match this template format.",
                        template_doc.content
                    )
                }
                std::result::Result::Ok(None) => {
                    tracing::warn!(
                        "Template document {} not found or access denied for user {}",
                        template_id,
                        user_id
                    );
                    return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                        error: "Template document not found or access denied".to_string(),
                    });
                }
                std::result::Result::Err(e) => {
                    tracing::error!(
                        "Database error when finding template document {} for user {}: {}",
                        template_id,
                        user_id,
                        e
                    );
                    return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                        error: "Database error while accessing template document".to_string(),
                    });
                }
            }
        }
        (Some(_), Some(_)) => {
            // Both provided - prefer transformation_prompt for backward compatibility
            tracing::warn!("Both transformation_prompt and template_document_id provided, using transformation_prompt");
            req.transformation_prompt.as_ref().unwrap().clone()
        }
        (None, None) => {
            tracing::warn!("Neither transformation_prompt nor template_document_id provided");
            return actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
                error: "Either transformation_prompt or template_document_id must be provided".to_string(),
            });
        }
    };

    // Transform the document using the standalone function
    match transform_document_content(
        pool.get_ref(),
        &source_document,
        &final_transformation_prompt,
        user_id,
        Some(req.target_title.clone()),
    ).await {
        Ok(transformed_document) => {
            let generation_time_ms = generation_start.elapsed().as_millis() as u64;
            
            tracing::info!(
                "Document transformation completed for user {} with source document {} in {}ms",
                user_id,
                req.source_document_id,
                generation_time_ms
            );

            let response = ContentGenerationResponse {
                document: transformed_document,
                source_document_id: Some(req.source_document_id),
                model_used: "gemini-2.5-pro".to_string(),
                generation_time_ms,
                has_provenance: true,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(
                "Document transformation failed for user {} with source document {}: {}",
                user_id,
                req.source_document_id,
                e
            );

            // Check for specific error types
            if e.contains("not found") || e.contains("access denied") {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "Source document not found or access denied".to_string(),
                })
            } else if e.contains("LLM") || e.contains("generation") {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Content generation service temporarily unavailable".to_string(),
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to transform document".to_string(),
                })
            }
        }
    }
}