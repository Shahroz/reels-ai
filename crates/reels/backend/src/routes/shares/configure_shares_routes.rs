//! Configures the routes for the shares API.
use actix_web::web;
use crate::routes::shares::create_share::create_share;
use crate::routes::shares::list_shares::list_shares;
use crate::routes::shares::delete_share::delete_share;

/// Mounts the share-related routes to the Actix web application.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to the Actix web `ServiceConfig`.
pub fn configure_shares_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create_share)      // POST /
            .service(list_shares)       // GET /
            .service(delete_share)      // DELETE /{share_id}
    );
} 