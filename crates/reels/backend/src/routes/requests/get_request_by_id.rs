//! Handler for retrieving a single user request by ID.
//!
//! Fetches a request record for the given ID and authenticated user.
use crate::db::requests::RequestRecord;
use crate::routes::error_response::ErrorResponse;
use tracing::instrument;

#[utoipa::path(
    get,
    path = "/api/requests/{id}",
    params(("id" = i32, Path, description = "ID of the request")),
    responses(
        (status = 200, description = "Request retrieved successfully", body = RequestRecord), // utoipa uses simple name
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Request not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Requests",
    security(("user_auth" = []))
)]
#[actix_web::get("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn get_request_by_id(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<i32>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let req_id = path.into_inner();

    let request =
        crate::queries::requests::get_request_by_id_and_user_id(&pool, req_id, user_id).await;

    match request {
        Ok(Some(rec)) => actix_web::HttpResponse::Ok().json(rec),
        Ok(None) => {
            actix_web::HttpResponse::NotFound().json(ErrorResponse { error: "Request not found".to_string() })
        }
        Err(e) => {
            log::error!(
                "Error retrieving request id {req_id} for user {user_id}: {e}"
            );
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to retrieve request".to_string() })
        }
    }
}
