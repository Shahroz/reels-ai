//! Configures Actix-web routes for bundle management.
//!
//! This function registers all bundle-related API endpoints.
//! It is intended to be mounted under a specific base path (e.g., `/api/bundles`) by the main application router.
//! Adheres to the project's Rust coding standards.

// Revision History
// - 2025-05-29T18:55:06Z @AI: Refactor to align with creatives module path configuration.
// - 2025-05-29T15:27:46Z @AI: Initial implementation.

use actix_web::web;
use super::{
    create_bundle_handler,
    list_bundles_handler,
    get_bundle_handler,
    update_bundle_handler,
    delete_bundle_handler,
};

/// Configures routes for bundle operations.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to Actix-web's `ServiceConfig`.
pub fn configure_bundle_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_bundle_handler::create_bundle)
        .service(list_bundles_handler::list_bundles)
        .service(get_bundle_handler::get_bundle_by_id)
        .service(update_bundle_handler::update_bundle)
        .service(delete_bundle_handler::delete_bundle);
}
