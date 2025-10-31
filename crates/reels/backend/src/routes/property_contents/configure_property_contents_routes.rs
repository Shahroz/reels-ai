//! Configuration for property contents routes.
//!
//! This module defines the Actix Web configuration function for registering
//! property contents routes under the `/api/property-contents` scope.

/// Configures property contents routes for the Actix Web application.
///
/// Registers all property contents endpoints under the `/api/property-contents` scope.
/// Routes are secured with JWT authentication and trial guard middleware.
///
/// # Arguments
///
/// * `cfg` - Actix Web service configuration to add routes to
pub fn configure_property_contents_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        crate::routes::property_contents::create_property_contents::create_property_contents,
    )
    .service(
        crate::routes::property_contents::create_property_contents_with_studio_journey::create_property_contents_with_studio_journey,
    );
} 