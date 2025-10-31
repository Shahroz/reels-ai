//! Handler for listing audit logs in the admin panel.
//!
//! This endpoint allows administrators to view and filter the complete audit trail.
//! Requires admin privileges and supports extensive filtering options including date ranges,
//! action types, and entity filters. Returns paginated results with metadata for building
//! admin interfaces with comprehensive audit trail visibility.
//!
//! Revision History:
//! - 2025-10-10: Initial creation with admin authorization middleware.

#[utoipa::path(
    get,
    path = "/api/admin/audit-logs",
    tag = "Admin",
    params(
        crate::routes::admin::audit_logs::list_audit_logs_params::ListAuditLogsParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved audit logs", body = crate::routes::admin::audit_logs::list_audit_logs_response::ListAuditLogsResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("")]
#[tracing::instrument(skip(pool, auth_claims, params))]
pub async fn list_audit_logs_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    params: actix_web::web::Query<crate::routes::admin::audit_logs::list_audit_logs_params::ListAuditLogsParams>,
) -> impl actix_web::Responder {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);

    match crate::queries::audit_logs::list_audit_logs(
        &pool,
        page,
        limit,
        params.admin_user_id,
        params.action_type.as_deref(),
        params.target_entity_type.as_deref(),
        params.target_entity_id,
        params.from_date,
        params.to_date,
    )
    .await
    {
        Ok((logs, total_count)) => {
            actix_web::HttpResponse::Ok().json(crate::routes::admin::audit_logs::list_audit_logs_response::ListAuditLogsResponse {
                items: logs,
                total_count,
                page,
                limit,
            })
        }
        Err(e) => {
            log::error!(
                "Failed to list audit logs for admin user {}: page={}, limit={}, filters={{admin_user_id: {:?}, action_type: {:?}, entity_type: {:?}, entity_id: {:?}, from: {:?}, to: {:?}}}, error: {}",
                auth_claims.user_id,
                page,
                limit,
                params.admin_user_id,
                params.action_type,
                params.target_entity_type,
                params.target_entity_id,
                params.from_date,
                params.to_date,
                e
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: String::from("Failed to retrieve audit logs. Please try again or contact support if the issue persists."),
            })
        }
    }
}

