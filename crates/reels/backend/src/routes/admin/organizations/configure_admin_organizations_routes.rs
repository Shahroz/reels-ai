//! Configures admin organization routes.
//!
//! This module registers all admin-level organization management endpoints.
//! These routes are mounted under /api/admin/organizations and require admin privileges.

pub fn configure_admin_organizations_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/organizations")
            .service(crate::routes::admin::organizations::list_all_organizations_handler::list_all_organizations_handler)
            .service(crate::routes::admin::organizations::admin_create_organization_handler::admin_create_organization_handler)
            .service(crate::routes::admin::organizations::admin_update_organization_handler::admin_update_organization_handler)
            .service(crate::routes::admin::organizations::update_organization_credits_handler::update_organization_credits_handler)
            .service(crate::routes::admin::organizations::admin_add_members_handler::admin_add_members_handler)
            .service(crate::routes::admin::organizations::admin_list_members_handler::admin_list_members_handler),
    );
}
