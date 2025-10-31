//! Handles the local file upload endpoint.
//!
//! This module contains the `POST /files/upload` endpoint, which accepts
//! `multipart/form-data` requests. It extracts the file, sends it for content
//! extraction, and then creates a new `Document` record from the result.

use crate::db::documents::Document;
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use anyhow::anyhow;
use futures_util::{future, StreamExt, TryStreamExt};

use crate::routes::files::file_upload_request::FileUpload;

/// The main handler for the `POST /files/upload` endpoint.
#[utoipa::path(
    post,
    path = "/api/files/upload",
    tag = "Files",
    request_body(content_type = "multipart/form-data", content = FileUpload),
    responses(
        (status = 200, description = "File(s) processed and document(s) created successfully.", body = Vec<Document>),
        (status = 400, description = "Bad request, e.g., no file provided."),
        (status = 401, description = "Unauthorized.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/upload")]
pub async fn upload_file(
    pool: web::Data<sqlx::PgPool>,
    auth: web::ReqData<crate::auth::tokens::Claims>,
    mut payload: Multipart,
) -> HttpResponse {
    let user_id = auth.user_id;
    let mut tasks = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let file_name = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .unwrap_or("unknown_file")
            .to_string();

        let mut file_data: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            if let Ok(data) = chunk {
                file_data.extend_from_slice(&data);
            } else {
                tracing::error!("Error reading file stream for file: {}", file_name);
                continue;
            }
        }

        if file_data.is_empty() {
            tracing::warn!("Uploaded file is empty, skipping: {}", file_name);
            continue;
        }

        let task = process_file_into_document(
            pool.get_ref().clone(),
            user_id,
            file_name,
            field.content_type().map(|m| m.to_string()).unwrap_or_default(),
            file_data,
        );
        tasks.push(task);
    }

    if tasks.is_empty() {
        return HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse::from("No file provided in the upload request."));
    }

    let results = future::join_all(tasks).await;
    let successful_documents: Vec<Document> = results.into_iter().filter_map(|res| res.ok()).collect();
    
    HttpResponse::Ok().json(successful_documents)
}

/// Asynchronous task to process a file and create a document.
async fn process_file_into_document(
    pool: sqlx::PgPool,
    user_id: uuid::Uuid,
    file_name: String,
    mime_type: String,
    file_data: Vec<u8>,
) -> Result<Document, anyhow::Error> {
    // 1. Extract text content
    let text_content = match crate::services::content_extraction::extract_text::extract_text(&file_data, &mime_type, &file_name).await {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Content extraction failed for file '{}' (user {}): {}", file_name, user_id, e);
            return Err(anyhow!(e));
        }
    };
    
    // 2. On success, create a new document record.
    let title = format!("Document Extracted from File: {file_name}");
    let sources = vec![format!("local_upload:{}", file_name)];
    
    let new_document = sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents (user_id, title, content, sources, status, is_public, is_task, include_research)
        VALUES ($1, $2, $3, $4, 'Completed', false, false, 'Never')
        RETURNING 
            id, user_id, title, content, sources, status, created_at, updated_at, 
            is_public, is_task, include_research as "include_research: _", collection_id
        "#,
        user_id,
        title,
        text_content,
        &sources,
    )
    .fetch_one(&pool)
    .await;

    match new_document {
        Ok(doc) => {
            tracing::info!("Successfully created document from file upload for user {}", user_id);
            Ok(doc)
        },
        Err(e) => {
            tracing::error!("Failed to save extracted document for user {}: {}", user_id, e);
            Err(e.into())
        }
    }
} 