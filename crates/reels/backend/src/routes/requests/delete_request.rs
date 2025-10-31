//! Handler for deleting a user request.
//!
//! Executes deletion of a request record for the given ID and authenticated user.

use crate::routes::error_response::ErrorResponse;
use tracing::instrument;

#[utoipa::path(
    delete,
    path = "/api/requests/{id}",
    params(("id" = i32, Path, description = "ID of the request to delete")),
    responses(
        (status = 204, description = "Request deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Request not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Requests",
    security(("user_auth" = []))
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn delete_request(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<i32>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let req_id = path.into_inner();

    let result =
        crate::queries::requests::delete_request_by_id_and_user_id(&pool, req_id, user_id).await;

    match result {
        Ok(affected_rows) if affected_rows > 0 => actix_web::HttpResponse::NoContent().finish(),
        Ok(_) => { // 0 rows affected
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Request not found or not owned by user"),
            })
        }
        Err(e) => {
            log::error!(
                "Error deleting request id {req_id} for user {user_id}: {e}"
            );
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to delete request"),
                },
            )
        }
    }
}
