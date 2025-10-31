//! Handler for batch deleting users via admin endpoint.
//!
//! This endpoint allows administrators to delete multiple users at once by providing
//! a list of user IDs. Includes safety checks to prevent deletion of admin users or
//! the requesting admin. Returns 207 Multi-Status with detailed success/failure results
//! for each user ID. The handler delegates to the service layer which handles the
//! complete business operation including transaction management and audit logging.

#[utoipa::path(
    delete,
    path = "/api/admin/users/batch",
    tag = "Admin",
    request_body = crate::routes::admin::users::batch_delete_users_request::BatchDeleteUsersRequest,
    responses(
        (status = 207, description = "Multi-Status - partial success", body = crate::routes::admin::users::batch_delete_users_response::BatchDeleteUsersResponse),
        (status = 400, description = "Bad request - invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::delete("/batch")]
#[tracing::instrument(skip(pool, auth_claims, payload))]
pub async fn batch_delete_users_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    payload: actix_web::web::Json<crate::routes::admin::users::batch_delete_users_request::BatchDeleteUsersRequest>,
) -> impl actix_web::Responder {
    match crate::queries::admin::users::services::batch_delete_users_service(
        pool.get_ref(),
        auth_claims.user_id,
        payload.user_ids.clone(),
    )
    .await
    {
        Ok(result) => {
            let success_dtos: Vec<crate::routes::admin::users::batch_delete_users_response::UserDeleteSuccess> = result
                .success
                .into_iter()
                .map(|s| crate::routes::admin::users::batch_delete_users_response::UserDeleteSuccess {
                    user_id: s.user_id,
                    email: s.email,
                })
                .collect();

            let failed_dtos: Vec<crate::routes::admin::users::batch_delete_users_response::UserDeleteFailure> = result
                .failed
                .into_iter()
                .map(|f| crate::routes::admin::users::batch_delete_users_response::UserDeleteFailure {
                    user_id: f.user_id,
                    reason: f.reason,
                })
                .collect();

            let response = crate::routes::admin::users::batch_delete_users_response::BatchDeleteUsersResponse {
                success: success_dtos,
                failed: failed_dtos,
            };

            actix_web::HttpResponse::MultiStatus().json(response)
        }
        Err(e) => {
            let error_msg = e.to_string();

            if error_msg.contains("must be provided") {
                log::warn!(
                    "Admin {} provided invalid input for batch deleting users: {}",
                    auth_claims.user_id,
                    error_msg
                );
                actix_web::HttpResponse::BadRequest().json(
                    crate::routes::error_response::ErrorResponse {
                        error: error_msg,
                    },
                )
            } else {
                log::error!(
                    "Admin {} failed to batch delete users: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: String::from("Failed to delete users."),
                    },
                )
            }
        }
    }
}

