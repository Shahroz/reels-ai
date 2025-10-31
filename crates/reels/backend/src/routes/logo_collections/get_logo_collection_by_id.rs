//! Handler for getting a logo collection by ID.
//!
//! GET /api/logo-collections/{id}

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::schemas::logo_collection_schemas::LogoCollectionResponse;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    get,
    path = "/api/logo-collections/{id}",
    tag = "Logo Collections",
    params(
        ("id" = Uuid, Path, description = "Logo collection ID")
    ),
    responses(
        (status = 200, description = "Success", body = LogoCollectionResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[get("/{id}")]
#[instrument(skip(pool, claims))]
pub async fn get_logo_collection_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let collection_id = path.into_inner();
    let user_id = claims.user_id;

    let result = crate::queries::logo_collections::get_logo_collection_by_id::get_logo_collection_by_id(
        pool.get_ref(),
        collection_id,
        user_id,
    )
    .await;

    match result {
        Ok(Some(collection)) => HttpResponse::Ok().json(collection),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Logo collection not found".to_string(),
        }),
        Err(err) => {
            log::error!("Database error getting logo collection {} for user {}: {:?}", collection_id, user_id, err);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to get logo collection".to_string(),
            })
        }
    }
}
