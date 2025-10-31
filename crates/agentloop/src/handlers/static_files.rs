//! Handles serving embedded static frontend assets.
//!
//! This module provides Actix-web handlers to serve files embedded
//! using the `rust-embed` crate, specifically the `Assets` struct defined
//! in `crate::assets`. It includes logic to handle file requests and
//! fall back to serving `index.html` for SPA routing.
//! Adheres strictly to the project's Rust coding standards.

use actix_web::{web, HttpRequest, HttpResponse, Responder, http::header::ContentType};
use rust_embed::RustEmbed;

/// Serves a requested static file from the embedded assets.
///
/// Attempts to retrieve the file specified by the `path` parameter from
/// the embedded `crate::assets::Assets`. If found, it serves the file
/// with the appropriate MIME type. If not found, it returns a 404 Not Found response.
///
/// # Arguments
/// * `path` - The path of the file requested, extracted from the URL.
///
/// # Returns
/// * An `HttpResponse` containing the file content or a 404 error.
async fn serve_static_file(path: web::Path<String>) -> HttpResponse {
    let requested_path = path.into_inner();
    match crate::assets::Assets::get(&requested_path) {
        Some(content) => {
            // Guess the MIME type based on the file extension.
            let mime_type = mime_guess::from_path(&requested_path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(content.data.into_owned())
        }
        None => {
            // Fallback to serving index.html if the specific file is not found,
            // suitable for Single Page Applications (SPAs).
            serve_index_fallback().await
        }
    }
}

/// Serves the `index.html` file as a fallback.
///
/// This is typically used for SPA routing where non-asset paths should
/// still load the main application shell.
///
/// # Returns
/// * An `HttpResponse` containing the `index.html` content or a 404 if `index.html` itself is missing.
async fn serve_index_fallback() -> HttpResponse {
    match crate::assets::Assets::get("index.html") {
        Some(content) => HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found: index.html missing"),
    }
}

/// Configures the routes for serving static files.
///
/// Adds routes to handle:
/// - `/`: Serves `index.html`.
/// - `/assets/{tail:.*}`: Serves files from the embedded `assets` directory.
/// - `/{tail:.*}`: A fallback route that serves `index.html` for SPA routing.
///
/// # Arguments
/// * `cfg` - Service config to add routes to.
pub fn configure_static_files(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(serve_index_fallback))
       .route("/index.html", web::get().to(serve_index_fallback)) // Explicit index.html route
       .route("/assets/{filename:.*}", web::get().to(serve_static_file))
       // Fallback for SPA routing - should be last
       .route("/{tail:.*}", web::get().to(serve_index_fallback));
}
