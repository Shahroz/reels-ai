//! Configures the Actix web service for API key management routes.
//!
//! This function registers the handlers for creating, listing, and deleting API keys.
//! It groups these handlers under the `/api/keys` scope (as defined in the parent module).
//! Ensures adherence to coding standards by containing only the configuration function.
//! Uses fully qualified paths for service registration and handlers.

/// Configures the routes for API key management.
pub fn configure_api_key_routes(cfg: &mut actix_web::web::ServiceConfig) {
    // Fully qualified path
    cfg.service(super::create_key_handler::create_key_handler) // Fully qualified path to handler
        .service(super::list_keys_handler::list_keys_handler) // Fully qualified path to handler
        .service(super::delete_key_handler::delete_key_handler); // Fully qualified path to handler
}
