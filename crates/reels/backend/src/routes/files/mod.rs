//! Defines the modules and configures the routes for the /files scope.
pub mod file_upload_request;
pub mod upload_file;
 
/// Configures the routes for the `/files` scope.
pub fn configure_files_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(upload_file::upload_file);
} 