//! Handler for retrieving a single audit log entry by ID.
//!
//! This endpoint allows administrators to view detailed information about a specific audit log.
//! Requires admin privileges and returns the complete audit log record.
//! Useful for drill-down views and detailed investigations of administrative actions.

#[utoipa::path(
    get,
    path = "/api/admin/audit-logs/{log_id}",
    tag = "Admin",
    params(
        ("log_id" = uuid::Uuid, Path, description = "The ID of the audit log to retrieve")
    ),
    responses(
        (status = 200, description = "Successfully retrieved audit log", body = crate::db::audit_logs::AuditLog),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Audit log not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/{log_id}")]
#[tracing::instrument(skip(pool, auth_claims))]
pub async fn get_audit_log_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    log_id: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    match crate::queries::audit_logs::get_audit_log_by_id(&pool, *log_id).await {
        Ok(log) => actix_web::HttpResponse::Ok().json(log),
        Err(e) => {
            if let Some(sqlx_error) = e.downcast_ref::<sqlx::Error>() {
                if matches!(sqlx_error, sqlx::Error::RowNotFound) {
                    log::warn!(
                        "Admin user {} attempted to access non-existent audit log: {}",
                        auth_claims.user_id,
                        log_id
                    );
                    return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                        error: std::format!("Audit log with ID {} not found.", log_id),
                    });
                }
            }
            log::error!(
                "Database error retrieving audit log {} for admin user {}: {}",
                log_id,
                auth_claims.user_id,
                e
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: String::from("Failed to retrieve audit log. Please try again or contact support if the issue persists."),
            })
        }
    }
}

