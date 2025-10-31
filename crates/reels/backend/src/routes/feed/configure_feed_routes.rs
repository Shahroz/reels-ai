//! Feed routes configuration

use actix_web::web;

pub fn configure_feed_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(super::create_post::create_feed_post)
        .service(super::get_feed::get_feed)
        .service(super::get_post::get_feed_post)
        .service(super::update_post::update_feed_post)
        .service(super::delete_post::delete_feed_post);
}

