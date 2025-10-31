//! Handler for the clone endpoint.
//!
//! Implements the POST /api/clone route, performing database logging and style cloning operations.
//!
//! Revision History
//! - 2025-04-21T15:01:17Z @AI: Refactored clone_style into its own file.

use crate::auth::tokens::Claims;
use crate::db::requests::{
    create_request, update_request_completion, update_request_status, CreateRequestArgs,
    UpdateRequestArgs,
};
use crate::routes::clone::{CloneRequest, CloneResponse};
use crate::style_cloning::style_replication::replicate_style;
use crate::zyte::zyte::ZyteClient;
use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use std::env;
use std::time::Instant;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/clone", // Combined scope and handler path
    tag = "Clone",
    request_body = CloneRequest,
    responses(
        (status = 200, description = "Style cloning successful", body = CloneResponse),
        (status = 400, description = "Invalid input (e.g., invalid mode)", body = inline(serde_json::Value)),
        (status = 401, description = "Unauthorized - Invalid or missing JWT"), // Handled by JwtMiddleware
        (status = 500, description = "Internal server error (DB or cloning failure)", body = inline(serde_json::Value))
    ),
    security(
        ("user_auth" = []) // Requires JWT authentication via middleware
    )
)]
#[post("")]
#[instrument(skip(pool, req, claims))]
pub async fn clone_style(
    pool: web::Data<PgPool>,
    req: web::Json<CloneRequest>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let start_time = Instant::now();
    log::info!(
        "Received clone request for URL: {} in mode: {} from user: {}",
        req.url,
        req.mode,
        claims.user_id
    );

    let create_args = CreateRequestArgs {
        url: Some(req.url.clone()),
        content_to_style: req.content_to_style.clone(),
        what_to_create: Some(format!("Clone style in '{}' mode", req.mode)),
        status: "processing".to_string(),
        user_id: Some(claims.user_id),
        visual_feedback: Some(req.mode.to_lowercase() == "visual"),
    };

    let request_id = match create_request(&pool, create_args).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Database error creating request record: {e}");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to initiate request logging."
            }));
        }
    };

    let mode = req.mode.to_lowercase();
    let result: Result<String, String> = if mode == "fast" {
        let api_key = env::var("ZYTE_API_KEY").unwrap_or_else(|_| "dummy_key".to_string());
        let zyte_client = ZyteClient::new(api_key);
        match zyte_client.extract_styles_with_fallback(&req.url).await {
            Ok(extracted_html) => {
                log::info!(
                    "Extracted {} character using inline styles from URL: {}",
                    extracted_html.len(),
                    req.url
                );
                let content = req.content_to_style.clone().unwrap_or_default();
                let replicated = replicate_style(&extracted_html, "", &content).await;
                if let Ok(c) = &replicated {
                    Ok(c.to_string())
                } else {
                    Ok("<html><body>Something went wrong :(</body></html>".to_string())
                }
            }
            Err(e) => Err(format!("Error extracting styles: {e}")),
        }
    } else if mode == "visual" {
        let content = req.content_to_style.clone().unwrap_or_default();
        match replicate_style(&req.url, "", content.as_str()).await {
            Ok(replica) => Ok(replica),
            Err(e) => Err(format!("Error in visual style replication: {e}")),
        }
    } else {
        Err(format!(
            "Invalid mode specified: {}. Use 'fast' or 'visual'.",
            req.mode
        ))
    };

    let execution_time_ms = start_time.elapsed().as_millis() as i32;

    match result {
        Ok(styled_content) => {
            let update_args = UpdateRequestArgs {
                compressed_style_website_content: None,
                compressed_output_html: Some(styled_content.as_bytes().to_vec()),
                status: "completed".to_string(),
                execution_time_ms: Some(execution_time_ms),
                credits_used: Some(1),
            };
            if let Err(e) = update_request_completion(&pool, request_id, update_args).await {
                log::error!("Database error updating request record on success: {e}");
            }
            HttpResponse::Ok().json(CloneResponse {
                styled_content,
                request_id,
            })
        }
        Err(error_message) => {
            log::error!(
                "Cloning failed for user {}: {}",
                claims.user_id,
                error_message
            );
            if let Err(e) = update_request_status(&pool, request_id, "failed").await {
                log::error!("Database error updating request status to failed: {e}");
            }
            if error_message.starts_with("Invalid mode") {
                HttpResponse::BadRequest().json(json!({
                   "error": error_message,
                   "request_id": request_id
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                   "error": error_message,
                   "request_id": request_id
                }))
            }
        }
    }
}
