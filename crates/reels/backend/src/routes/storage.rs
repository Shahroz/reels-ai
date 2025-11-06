//! Route handler for serving locally stored video files.
//!
//! This module provides an endpoint to serve generated reels from local storage
//! with support for HTTP range requests for video streaming.

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::path::PathBuf;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

/// Parses a Range header string (e.g., "bytes=0-1023") and returns (start, end) tuple.
fn parse_range_header(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    // Remove "bytes=" prefix
    let range_str = range_str.strip_prefix("bytes=")?;
    
    // Split by "-"
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let start_str = parts[0].trim();
    let end_str = parts[1].trim();
    
    let start = if start_str.is_empty() {
        0
    } else {
        start_str.parse::<u64>().ok()?
    };
    
    let end = if end_str.is_empty() {
        file_size - 1
    } else {
        end_str.parse::<u64>().ok()?
    };
    
    // Validate range
    if start > end || end >= file_size {
        return None;
    }
    
    Some((start, end))
}

/// Serves a video file from local storage with support for HTTP range requests.
///
/// Path: GET /storage/reels/{reel_id}.mp4
///
/// Supports Range requests for video streaming:
/// - Range: bytes=0-1023 (first 1024 bytes)
/// - Range: bytes=1024- (from byte 1024 to end)
/// - Range: bytes=-1024 (last 1024 bytes)
///
/// Returns the video file if it exists, or 404 if not found.
pub async fn serve_video(path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let relative_path = path.into_inner();
    
    // Get storage base path from environment or use default
    let storage_base = env::var("REELS_STORAGE_PATH")
        .unwrap_or_else(|_| "storage/reels".to_string());
    
    // The path will be something like "reels/{reel_id}.mp4"
    // Since storage_base is "storage/reels", we need to extract just the filename
    // from "reels/{reel_id}.mp4" -> "{reel_id}.mp4"
    let file_path = if relative_path.starts_with("reels/") {
        let filename = relative_path.strip_prefix("reels/").unwrap_or(&relative_path);
        PathBuf::from(&storage_base).join(filename)
    } else {
        // If it doesn't start with "reels/", use it as-is (handles edge cases)
        PathBuf::from(&storage_base).join(&relative_path)
    };
    
    // Security: Ensure the path is within the storage directory
    let storage_base_path = match PathBuf::from(&storage_base).canonicalize() {
        Ok(p) => p,
        Err(_) => PathBuf::from(&storage_base),
    };
    
    let canonical_file_path = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            log::debug!("File not found: {}", file_path.display());
            return HttpResponse::NotFound().body("File not found");
        }
    };
    
    // Verify the file is within the storage directory (prevent path traversal)
    if !canonical_file_path.starts_with(&storage_base_path) {
        log::warn!("Path traversal attempt detected: {}", canonical_file_path.display());
        return HttpResponse::Forbidden().body("Access denied");
    }
    
    // Check if file exists and serve it
    if !file_path.exists() {
        log::debug!("File does not exist: {}", file_path.display());
        return HttpResponse::NotFound().body("File not found");
    }

    // Get file metadata
    let file_size = match fs::metadata(&canonical_file_path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            log::error!("Failed to get file metadata {}: {}", canonical_file_path.display(), e);
            return HttpResponse::InternalServerError().body("Failed to read file");
        }
    };

    // Open file for reading
    let mut file = match File::open(&canonical_file_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to open file {}: {}", canonical_file_path.display(), e);
            return HttpResponse::InternalServerError().body("Failed to read file");
        }
    };

    // Check for Range header for video streaming
    if let Some(range_header) = req.headers().get("range") {
        if let Ok(range_str) = range_header.to_str() {
            if let Some(range) = parse_range_header(range_str, file_size) {
                let (start, end) = range;
                let content_length = end - start + 1;

                // Seek to start position
                if file.seek(SeekFrom::Start(start)).is_err() {
                    log::error!("Failed to seek to position {}", start);
                    return HttpResponse::InternalServerError().body("Failed to read file");
                }

                // Read the requested byte range
                let mut buffer = vec![0u8; content_length as usize];
                if file.read_exact(&mut buffer).is_err() {
                    log::error!("Failed to read file range");
                    return HttpResponse::InternalServerError().body("Failed to read file");
                }

                log::debug!("Serving range {}-{} of file: {}", start, end, canonical_file_path.display());
                
                // Format Content-Range header: bytes start-end/total
                let content_range = format!("bytes {}-{}/{}", start, end, file_size);
                
                return HttpResponse::PartialContent()
                    .content_type("video/mp4")
                    .insert_header(("Accept-Ranges", "bytes"))
                    .insert_header(("Content-Range", content_range))
                    .insert_header(("Content-Length", content_length))
                    .body(buffer);
            }
        }
    }

    // No range header or invalid range - serve entire file
    // For large files, we should still stream, but for now we'll read it
    // In production, consider using streaming for large files
    match fs::read(&canonical_file_path) {
        Ok(content) => {
            log::debug!("Serving full file: {}", canonical_file_path.display());
            HttpResponse::Ok()
                .content_type("video/mp4")
                .insert_header(("Accept-Ranges", "bytes"))
                .insert_header(("Content-Length", file_size))
                .body(content)
        }
        Err(e) => {
            log::error!("Failed to read file {}: {}", canonical_file_path.display(), e);
            HttpResponse::InternalServerError().body("Failed to read file")
        }
    }
}

/// Configures storage routes.
pub fn configure_storage_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/storage/{tail:.*}", web::get().to(serve_video));
}

