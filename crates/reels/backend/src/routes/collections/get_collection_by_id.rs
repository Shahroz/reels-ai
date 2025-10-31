//! Handler for fetching a collection by ID.
//!
//! GET /api/collections/{id}

use crate::db::collections::Collection;
use crate::routes::error_response::ErrorResponse;
use tracing::instrument;

#[utoipa::path(
    get,
    path = "/api/collections/{id}",
    tag = "Collections",
    responses(
        (status = 200, description = "Collection found", body = Collection),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::get("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn get_collection_by_id(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: crate::auth::tokens::Claims,
    id: actix_web::web::Path<sqlx::types::Uuid>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    match crate::queries::collections::get_collection_with_sharing::get_collection_with_sharing(pool.get_ref(), *id, user_id).await {
        Ok(Some(item)) => actix_web::HttpResponse::Ok().json(item),
        Ok(None) => {
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: "Collection not found or access denied".to_string(),
            })
        }
        Err(e) => actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: e.to_string(),
            },
        ),
    }
}
