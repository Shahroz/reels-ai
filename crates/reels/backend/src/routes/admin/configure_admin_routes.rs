//! Configures admin-specific route handlers.
//!
//! Registers each admin endpoint with Actix-web, typically under the `/api/admin` scope.
use actix_web::web;

/// Sets up endpoints for administrative operations.
///
/// # Arguments
///
/// * `cfg` - ServiceConfig to configure.
pub fn configure_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .configure(crate::routes::admin::users::configure_users_routes::configure_users_routes),
    );
    cfg.configure(
        crate::routes::admin::audit_logs::configure_audit_logs_routes::configure_audit_logs_routes,
    );
    cfg.configure(crate::routes::admin::organizations::configure_admin_organizations_routes::configure_admin_organizations_routes);
    cfg.service(
        web::scope("/unlimited-access")
            .configure(crate::routes::admin::unlimited_access::configure_unlimited_access_routes::configure_unlimited_access_routes),
    );
}
