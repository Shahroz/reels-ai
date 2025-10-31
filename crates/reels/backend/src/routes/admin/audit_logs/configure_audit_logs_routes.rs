//! Configures audit logs routes for the admin API.
//!
//! This function registers all audit log-related endpoints under the /admin/audit-logs scope.
//! All routes require admin authentication via the Claims extractor.

pub fn configure_audit_logs_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/audit-logs")
            .service(crate::routes::admin::audit_logs::list_audit_logs_handler::list_audit_logs_handler)
            .service(crate::routes::admin::audit_logs::get_audit_log_handler::get_audit_log_handler),
    );
}

