//! Handler for deleting a document from Content Studio.
//!
//! This deletes the document, its studio nodes, and all related provenance edges.
//! Similar to asset deletion in Image Studio but adapted for documents.

use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;
use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::queries::studio_cleanup::cleanup_studio_relationships;

#[utoipa::path(
    delete,
    path = "/api/content-studio/document/{document_id}",
    tag = "Content Studio",
    params(
        ("document_id" = String, Path, description = "Document ID to delete")
    ),
    responses(
        (status = 204, description = "Document deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Document access denied"),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[actix_web::delete("/document/{document_id}")]
#[instrument(skip(pool, claims))]
pub async fn delete_content_studio_document(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let document_id = path.into_inner();
    let user_id = claims.user_id;

    tracing::info!("🗑️  DELETE_CONTENT_STUDIO_DOCUMENT: Deleting document {} for user {}", document_id, user_id);

    // Start transaction for atomic deletion
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin transaction for document deletion: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to process deletion request"));
        }
    };

    // 1. Check if document exists and user has permission
    let document = match sqlx::query!(
        "SELECT id, user_id, title FROM documents WHERE id = $1",
        document_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            let _ = tx.rollback().await;
            return HttpResponse::NotFound().json(ErrorResponse::from("Document not found"));
        }
        Err(e) => {
            tracing::error!("Failed to fetch document {} for deletion: {}", document_id, e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve document"));
        }
    };

    // 2. Permission check - only owner can delete (user_id is optional in documents table)
    match document.user_id {
        Some(owner_id) if owner_id == user_id => {
            // User owns the document, proceed with deletion
        }
        Some(owner_id) => {
            tracing::warn!("User {} attempted to delete document {} owned by user {}", user_id, document_id, owner_id);
            let _ = tx.rollback().await;
            return HttpResponse::NotFound().json(ErrorResponse::from("Document not found or you do not own this document"));
        }
        None => {
            tracing::warn!("User {} attempted to delete public document {}", user_id, document_id);
            let _ = tx.rollback().await;
            return HttpResponse::NotFound().json(ErrorResponse::from("Cannot delete public documents"));
        }
    }

    tracing::info!("✅ PERMISSION_CHECK passed for document {}", document_id);

    // 3. Clean up all studio relationships (nodes, edges, journeys) using centralized cleanup
    let cleanup_summary = match cleanup_studio_relationships(&mut tx, document_id, "document").await {
        Ok(summary) => {
            tracing::info!(
                "🔗 STUDIO_CLEANUP_COMPLETE: {} nodes, {} orphaned nodes, {} edges, {} journeys deleted for document {}",
                summary.deleted_nodes, summary.deleted_orphaned_nodes, summary.deleted_edges, summary.deleted_journeys, document_id
            );
            summary
        }
        Err(e) => {
            tracing::error!("Failed to clean up studio relationships for document {}: {}", document_id, e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to clean up studio relationships"));
        }
    };

    // 4. Delete the document itself
    let document_deleted = match sqlx::query!(
        "DELETE FROM documents WHERE id = $1",
        document_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(result) => {
            let rows = result.rows_affected();
            tracing::info!("📄 DOCUMENT_DELETED: {} document record deleted for {}", rows, document_id);
            rows > 0
        }
        Err(e) => {
            tracing::error!("Failed to delete document {}: {}", document_id, e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to delete document"));
        }
    };

    // 5. Commit transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit document deletion transaction for {}: {}", document_id, e);
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to complete deletion"));
    }

    if document_deleted {
        tracing::info!(
            "✅ DOCUMENT_DELETION_COMPLETE: Document {} successfully deleted with {} studio nodes, {} orphaned nodes, {} provenance edges, {} journeys",
            document_id, cleanup_summary.deleted_nodes, cleanup_summary.deleted_orphaned_nodes, cleanup_summary.deleted_edges, cleanup_summary.deleted_journeys
        );
        HttpResponse::NoContent().finish()
    } else {
        tracing::warn!("Document {} was not found during deletion", document_id);
        HttpResponse::NotFound().json(ErrorResponse::from("Document not found"))
    }
}
