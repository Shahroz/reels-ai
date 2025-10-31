//! Handler for deleting a collection.
//!
//! DELETE /api/collections/{id}

use crate::routes::error_response::ErrorResponse;
use tracing::instrument;

#[utoipa::path(
    delete,
    path = "/api/collections/{id}",
    tag = "Collections",
    responses(
        (status = 204, description = "No Content"),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool))]
pub async fn delete_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    id: actix_web::web::Path<sqlx::types::Uuid>,
) -> impl actix_web::Responder {
    match crate::queries::collections::delete_collection::delete_collection(pool.get_ref(), *id).await {
        Ok(_) => actix_web::HttpResponse::NoContent().finish(),
        Err(e) => actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: e.to_string(),
            },
        ),
    }
}
