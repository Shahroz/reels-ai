//! Configures the routes for the Studio Journey Shares API.
use actix_web::web;

pub fn configure_studio_journey_shares_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(super::create_share::create_share)
        .service(super::get_share::get_share)
        .service(super::delete_share::delete_share);
}