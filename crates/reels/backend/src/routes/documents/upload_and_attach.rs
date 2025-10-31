//! Handler for synchronously uploading files and creating documents.
//!
//! This endpoint accepts a multipart/form-data request, processes each file,
//! extracts its content, creates a new document for each, and returns the full
//! created document objects. This is used for workflows where the frontend needs
//! to immediately use the created documents, such as attaching them
//! to a research agent context.

use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;

use crate::auth::tokens::Claims;
use crate::db::documents::Document;

#[derive(Serialize, utoipa::ToSchema)]
pub struct UploadAndAttachResponse {
    created_documents: Vec<Document>,
}

#[utoipa::path(
    post,
    path = "/api/documents/upload-and-attach",
    tag = "Documents",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Files processed and documents created.", body = UploadAndAttachResponse),
        (status = 400, description = "Bad request, e.g., no files provided."),
        (status = 401, description = "Unauthorized.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/upload-and-attach")]
pub async fn upload_and_attach(
    pool: web::Data<sqlx::PgPool>,
    auth: web::ReqData<Claims>,
    mut payload: Multipart,
) -> impl Responder {
    let user_id = auth.user_id;
    let is_admin = auth.is_admin;
    let mut created_documents = Vec::new();
    let mut is_public_requested = false;

    while let Ok(Some(mut field)) = payload.try_next().await {
        // Check if this is the is_public form field
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(name) = content_disposition.get_name() {
                if name == "is_public" {
                    // Read the is_public value
                    let mut field_data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        match chunk {
                            Ok(data) => field_data.extend_from_slice(&data),
                            Err(e) => {
                                tracing::error!("Error reading is_public field: {}", e);
                                break;
                            }
                        }
                    }
                    
                    if let Ok(value_str) = String::from_utf8(field_data) {
                        is_public_requested = value_str.trim().to_lowercase() == "true";
                        tracing::debug!("is_public field received: {}", is_public_requested);
                    }
                    continue;
                }
            }
        }

        let file_name = match field.content_disposition().and_then(|cd| cd.get_filename()) {
            Some(name) => name.to_string(),
            None => {
                tracing::warn!("Skipping multipart field without a filename.");
                continue;
            }
        };

        let mut file_data: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => file_data.extend_from_slice(&data),
                Err(e) => {
                    tracing::error!("Error reading stream for file {}: {}", file_name, e);
                    continue; // Skip to the next file
                }
            }
        }

        if file_data.is_empty() {
            tracing::warn!("Skipping empty file: {}", file_name);
            continue;
        }

        let mime_type = field.content_type().map(|m| m.to_string()).unwrap_or_default();

        match crate::services::content_extraction::extract_text::extract_text(&file_data, &mime_type, &file_name).await {
            Ok(text_content) => {
                let title = format!("Document Extracted from File: {file_name}");
                let sources = vec![format!("local_upload:{}", file_name)];
                
                // Handle is_public field with admin check (similar to create_document.rs)
                let is_public = if is_public_requested {
                    if !is_admin {
                        tracing::warn!(
                            "Non-admin user {} attempted to upload public document",
                            user_id
                        );
                        // For file upload, we'll just ignore the request and make it private
                        // rather than failing the entire upload
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };

                // For public documents, set user_id to NULL so they're accessible to all users
                let document_user_id = if is_public { None } else { Some(user_id) };
                
                let insert_result = sqlx::query_as!(
                    Document,
                    r#"
                    INSERT INTO documents (user_id, title, content, sources, status, is_public, is_task, include_research)
                    VALUES ($1, $2, $3, $4, 'Completed', $5, false, 'Never')
                    RETURNING 
                        id,
                        user_id,
                        title,
                        content,
                        sources,
                        status,
                        created_at,
                        updated_at,
                        is_public,
                        is_task,
                        include_research as "include_research: _",
                        collection_id
                    "#,
                    document_user_id,
                    title,
                    text_content,
                    &sources,
                    is_public,
                )
                .fetch_one(pool.get_ref())
                .await;

                match insert_result {
                    Ok(document) => {
                        created_documents.push(document);
                    }
                    Err(e) => {
                        tracing::error!("Failed to save extracted document for file {}: {}", file_name, e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Content extraction failed for file '{}' (user {}): {}", file_name, user_id, e);
            }
        }
    }

    HttpResponse::Ok().json(UploadAndAttachResponse { created_documents })
}
