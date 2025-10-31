//! Configures user-related admin routes.
//!
//! Registers handlers for user management under the `/api/admin/users` scope.
use actix_web::web;

/// Sets up endpoints for admin-level user operations.
///
/// # Arguments
///
/// * `cfg` - ServiceConfig to configure.
pub fn configure_users_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(crate::routes::admin::users::create_user_handler::create_user_handler);
    cfg.service(crate::routes::admin::users::activate_user_handler::activate_user_handler);
    cfg.service(crate::routes::admin::users::deactivate_user_handler::deactivate_user_handler);
    cfg.service(crate::routes::admin::users::impersonate_user_handler::impersonate_user_handler);
    cfg.service(crate::routes::admin::users::list_users_handler::list_users_handler);
    cfg.service(crate::routes::admin::users::update_user_handler::update_user_handler);
    cfg.service(crate::routes::admin::users::update_user_status_handler::update_user_status_handler);
    cfg.service(crate::routes::admin::users::update_user_credits_handler::update_user_credits_handler);
    cfg.service(crate::routes::admin::users::batch_create_users_handler::batch_create_users_handler);
    cfg.service(crate::routes::admin::users::batch_delete_users_handler::batch_delete_users_handler);
}
