//! Route handler for downloading files from storage.
//!
//! Supports downloading files from Google Cloud Storage or local file system.
//! Provides secure file access with path traversal protection.

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::path::PathBuf;
use std::env;

/// Handles GET requests to download files from storage.
///
/// # Arguments
///
/// * `file_path` - Path parameter containing the file identifier/path
/// * `req` - HTTP request (used for headers if needed in the future)
/// * `gcs_client` - Optional GCS client from app_data
///
/// # Returns
///
/// * `HttpResponse` - OK (200) with file content on success,
///   or NotFound (404) if file doesn't exist,
///   or InternalServerError (500) on download failure.
pub async fn download_file(
    file_path: web::Path<String>,
    _req: HttpRequest,
    gcs_client: Option<web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>>,
) -> impl Responder {
    let file_path_str = file_path.into_inner();
    log::debug!("Received download_file request for: {}", file_path_str);

    // Check if GCS bucket is configured and GCS client is available
    let gcs_bucket = env::var("GCS_BUCKET_NAME").ok();
    
    // Try GCS if both bucket name and client are available
    if let (Some(bucket_name), Some(client_data)) = (gcs_bucket, gcs_client.as_ref()) {
        // Download from GCS using the reels GCS service
        // web::Data<T> contains Arc<T>, so get the inner Arc
        let client = client_data.get_ref();
        match download_from_gcs(&bucket_name, &file_path_str, client).await {
            Ok((content, content_type)) => {
                return HttpResponse::Ok()
                    .content_type(content_type)
                    .insert_header((
                        actix_web::http::header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", 
                            file_path_str.split('/').last().unwrap_or("file")
                        ),
                    ))
                    .body(content);
            }
            Err(e) => {
                log::error!("Failed to download file from GCS: {}", e);
                if e.to_string().contains("not found") || e.to_string().contains("404") {
                    return HttpResponse::NotFound()
                        .json(serde_json::json!({ "error": "File not found" }));
                } else {
                    // On GCS error, fall through to try local storage
                    log::warn!("GCS download failed, falling back to local storage: {}", e);
                }
            }
        }
    }
    
    // Fallback: try local file system (for development/testing or when GCS is unavailable)
    match download_from_local(&file_path_str).await {
        Ok((content, content_type)) => {
            HttpResponse::Ok()
                .content_type(content_type)
                .insert_header((
                    actix_web::http::header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", 
                        file_path_str.split('/').last().unwrap_or("file")
                    ),
                ))
                .body(content)
        }
        Err(e) => {
            log::error!("Failed to download file: {}", e);
            HttpResponse::NotFound()
                .json(serde_json::json!({ "error": "File not found" }))
        }
    }
}

/// Downloads a file from Google Cloud Storage using the reels GCS service.
async fn download_from_gcs(
    bucket_name: &str, 
    object_name: &str,
    gcs_client: &std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>,
) -> anyhow::Result<(Vec<u8>, String)> {
    // Download the object
    let content = gcs_client
        .download_object_as_bytes(bucket_name, object_name)
        .await?;
    
    // Try to determine content type from object name
    let content_type = mime_guess::from_path(object_name)
        .first_or_octet_stream()
        .to_string();
    
    Ok((content, content_type))
}

/// Downloads a file from the local file system.
/// Note: This is a fallback for development/testing only.
async fn download_from_local(file_path: &str) -> anyhow::Result<(Vec<u8>, String)> {
    // Security: Prevent path traversal attacks
    let mut sanitized_path = file_path
        .replace("..", "")
        .replace("//", "/");
    if sanitized_path.starts_with('/') {
        sanitized_path.remove(0);
    }
    
    // Determine base storage directory (from env or default)
    let storage_dir = env::var("STORAGE_DIR")
        .unwrap_or_else(|_| "storage".to_string());
    
    let storage_dir_path = PathBuf::from(&storage_dir);
    let full_path = storage_dir_path.join(&sanitized_path);
    
    // Ensure the file is within the storage directory (additional security check)
    let storage_path = storage_dir_path.canonicalize()
        .map_err(|e| anyhow::anyhow!("Storage directory not found: {}", e))?;
    
    let canonical_file_path = full_path.canonicalize()
        .map_err(|e| anyhow::anyhow!("File not found: {}", e))?;
    
    if !canonical_file_path.starts_with(&storage_path) {
        return Err(anyhow::anyhow!("Path traversal detected"));
    }
    
    // Read the file
    let content = tokio::fs::read(&canonical_file_path).await
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
    
    // Determine content type
    let content_type = mime_guess::from_path(&canonical_file_path)
        .first_or_octet_stream()
        .to_string();
    
    Ok((content, content_type))
}

/// Configures file download routes.
pub fn configure_download_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/files/{file_path:.*}", web::get().to(download_file));
}

