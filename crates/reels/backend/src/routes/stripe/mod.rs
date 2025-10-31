pub mod webhooks;
pub mod analytics_helper;

use actix_web::web;

pub fn configure_stripe_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stripe")
            .configure(webhooks::configure_routes)
    );
} 