//! Configuration for unlimited access admin routes.
//!
//! This module configures all admin endpoints related to managing
//! unlimited credit access grants. Includes grant, revoke, and list operations.
//! All routes require admin authentication.

pub fn configure_unlimited_access_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("")
            .service(crate::routes::admin::unlimited_access::grant_unlimited_to_user_handler::grant_unlimited_to_user_handler)
            .service(crate::routes::admin::unlimited_access::revoke_unlimited_from_user_handler::revoke_unlimited_from_user_handler)
            .service(crate::routes::admin::unlimited_access::list_unlimited_grants_handler::list_unlimited_grants_handler)
    );
}

