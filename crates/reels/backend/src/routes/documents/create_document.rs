//! Handler for creating a new document task.
//!
//! Accepts a JSON body with title, content, and sources. Inserts a new record
//! with status "Pending", and returns the created Document entry on success.

use crate::auth::tokens::Claims;
use crate::db::documents::Document;
use crate::db::document_research_usage::DocumentResearchUsage; // Added for type hint
use crate::routes::documents::responses::DocumentResponse;
use crate::routes::error_response::ErrorResponse;
use crate::routes::documents::create_document_request::CreateDocumentRequest; // Corrected import
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/documents",
    tag = "Documents",
    request_body = CreateDocumentRequest,
    responses(
        (status = 201, description = "Document created", body = DocumentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation Error", body = ErrorResponse), // Added 422 response
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[post("")] // Ensure the route macro is correct if it wasn't root before
pub async fn create_document(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateDocumentRequest>,
) -> impl Responder {
    // Validate the request body
    if let Err(validation_errors) = req.validate() {
        log::warn!(
            "Validation failed for create document request by user {}: {}",
            claims.user_id,
            validation_errors
        );
        return HttpResponse::UnprocessableEntity().json(ErrorResponse {
            error: format!("Validation failed: {validation_errors}"),
        });
    }

    let user_id = claims.user_id;
    let title = req.title.clone();
    let content = req.content.clone();
    let sources = req.sources.clone().unwrap_or_default();

    // Handle is_public field with admin check from origin/main
    let is_public = if let Some(requested_public) = req.is_public {
        if requested_public && !claims.is_admin {
            log::warn!(
                "Non-admin user {} attempted to create public document",
                claims.user_id
            );
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Only administrators can create public documents".into(),
            });
        }
        requested_public
    } else {
        false // Default for new documents
    };

    let is_task = req.is_task.unwrap_or(false);
    let include_research = req.include_research.clone();

    // For public documents, set user_id to NULL so they're accessible to all users
    let document_user_id = if is_public { None } else { Some(user_id) };

    // Fetch user's email for creator_email field from HEAD, but without a transaction
    let user_email_result = sqlx::query_scalar!(
        "SELECT email FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    let creator_email = match user_email_result {
        Ok(Some(email)) => email,
        Ok(None) => {
            log::error!("User {user_id} not found when fetching email for new document.");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user details for document creation".into(),
            });
        }
        Err(e) => {
            log::error!("Error fetching user email for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user details for document creation".into(),
            });
        }
    };

    // Use query! from HEAD to fetch raw record, then build Document struct, combining approaches
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO documents (user_id, title, content, sources, status, is_public, is_task, include_research)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
            id, user_id, title, content, sources, status, created_at, updated_at, 
            is_public, is_task, include_research
        "#,
        document_user_id,
        title,
        content,
        &sources,
        "Pending",
        is_public,
        is_task,
        include_research as Option<DocumentResearchUsage>
    )
    .fetch_one(pool.get_ref())
    .await;

    match insert_result {
        Ok(record) => {
            // Manually construct the full Document response, combining fields from both branches
            let document = Document {
                id: record.id,
                user_id: record.user_id,
                title: record.title,
                content: record.content,
                sources: record.sources,
                status: record.status,
                created_at: record.created_at,
                updated_at: record.updated_at,
                is_public: record.is_public,
                is_task: record.is_task,
                include_research: record.include_research.map(|s| s.parse().unwrap_or(DocumentResearchUsage::TaskDependent)),
                collection_id: std::option::Option::None, // New documents start without collection assignment
            };

            let response = DocumentResponse {
                document,
                creator_email: Some(creator_email),
                current_user_access_level: Some("owner".to_string()),
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!("Error creating document for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create document".into(),
            })
        }
    }
}
