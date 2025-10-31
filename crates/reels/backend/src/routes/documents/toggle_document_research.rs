//! Handles toggling the research inclusion setting for a document.
//!
//! This operation allows a user to quickly toggle a document's research
//! inclusion setting from Always to null (not included) and vice versa.
//! It reuses the existing update infrastructure for permissions and validation.

use actix_web::{patch, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::auth::tokens::Claims;
use crate::db::document_research_usage::DocumentResearchUsage;
use crate::routes::error_response::ErrorResponse;
use crate::routes::documents::responses::DocumentResponse;
use crate::routes::documents::update_document_error::UpdateDocumentError;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ToggleDocumentResearchRequest {
    /// New research inclusion setting
    #[schema(example = "Always")]
    pub include_research: Option<DocumentResearchUsage>,
}

#[utoipa::path(
    patch,
    path = "/api/documents/{id}/toggle-research",
    tag = "Documents",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    request_body = ToggleDocumentResearchRequest,
    responses(
        (status = 200, description = "Document research setting updated", body = DocumentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User lacks edit permissions", body = ErrorResponse),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[patch("/{id}/toggle-research")]
pub async fn toggle_document_research(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
    req: web::Json<ToggleDocumentResearchRequest>,
) -> impl Responder {
   let document_id = path.into_inner();
   let authenticated_user_id = claims.user_id;
   
    // Check permissions
    let permission_result = match crate::queries::documents::check_update_permissions::check_update_permissions(
        &pool,
        document_id,
        authenticated_user_id,
        claims.is_admin,
    ).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("Permission check failed for document {document_id}: {e}");
            return match e {
                sqlx::Error::RowNotFound => UpdateDocumentError::NotFound("Document not found".to_string()),
                _ => UpdateDocumentError::DatabaseError("Permission check failed".to_string()),
            }.to_http_response();
        }
    };
    
    // Update document using existing transaction logic
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction: {e}");
            return UpdateDocumentError::DatabaseError("Failed to start database transaction".to_string())
                .to_http_response();
        }
    };
    
    // Use a direct SQL update that can properly set include_research to NULL
    let record = match sqlx::query!(
        r#"
        UPDATE documents SET
            include_research = $2,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id, user_id, title, content, sources, status, created_at, updated_at,
            is_public, is_task, include_research, collection_id
        "#,
       document_id,
       req.include_research.clone() as Option<crate::db::document_research_usage::DocumentResearchUsage>
   ).fetch_one(&mut *tx).await {
       Ok(record) => record,
       Err(e) => {
           log::error!("Failed to update document {document_id}: {e}");
            let _ = tx.rollback().await;
            return UpdateDocumentError::DatabaseError("Failed to update document".to_string())
                .to_http_response();
        }
    };
    
    let updated_document = crate::db::documents::Document {
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
        include_research: record.include_research.map(crate::db::document_research_usage::DocumentResearchUsage::from),
        collection_id: record.collection_id,
    };
    
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction for document update {document_id}: {e}");
        return UpdateDocumentError::DatabaseError("Failed to finalize document update".to_string())
            .to_http_response();
    }
    
    // Return response
    let response = DocumentResponse {
        document: updated_document,
        creator_email: permission_result.creator_email,
        current_user_access_level: permission_result.effective_access_level,
    };
    
    HttpResponse::Ok().json(response)
}
