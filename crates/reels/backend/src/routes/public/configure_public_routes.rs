//! Configures public API routes.
use actix_web::web;

/// Sets up endpoints that do not require authentication.
pub fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/journeys")
            .service(super::view_journey::view_journey),
    );
}