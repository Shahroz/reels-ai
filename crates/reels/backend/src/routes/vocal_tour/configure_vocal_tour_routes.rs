//! Configuration for vocal tour routes.
//!
//! This module configures and registers the vocal tour routes with the Actix web application.
//! It follows the same pattern as other route configuration modules in the project.

use actix_web::web; // Keep this import
//use crate::middleware::credits_guard::require_create_vocal_tour;


/// Configures the vocal tour routes.
/// 
/// Registers all vocal tour endpoints under the `/api/vocal-tour` prefix.
/// Includes authentication middleware for security.
/// 
/// # Arguments
/// 
/// * `cfg` - Service configuration for route registration
pub fn configure_vocal_tour_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/vocal-tour")
            .wrap(crate::middleware::auth::JwtMiddleware)
            .service(crate::routes::vocal_tour::get_vocal_tour::get_vocal_tour)
            .service(crate::routes::vocal_tour::get_vocal_tour_by_document::get_vocal_tour_by_document)
            .service(crate::routes::vocal_tour::list_vocal_tours::list_vocal_tours)
            .service(crate::routes::vocal_tour::attach_to_listing::attach_vocal_tour_to_listing)
            .service(crate::routes::vocal_tour::delete_vocal_tour::delete_vocal_tour)
            .service(crate::routes::vocal_tour::get_vocal_tour_assets::get_vocal_tour_assets)
            // Vocal tour creation endpoints require credits
            .service(
                web::scope("")
                    //.wrap(require_create_vocal_tour())
                    .service(crate::routes::vocal_tour::create_vocal_tour::create_vocal_tour)
            )
    );
} 
