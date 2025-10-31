//! Handler for updating a logo collection.
//!
//! PUT /api/logo-collections/{id}

use actix_web::{put, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::db::logo_collection::LogoCollection;
use crate::schemas::logo_collection_schemas::UpdateLogoCollectionRequest;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    put,
    path = "/api/logo-collections/{id}",
    tag = "Logo Collections",
    params(
        ("id" = Uuid, Path, description = "Logo collection ID")
    ),
    request_body = UpdateLogoCollectionRequest,
    responses(
        (status = 200, description = "Success", body = LogoCollection),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[put("/{id}")]
#[instrument(skip(pool, payload, claims))]
pub async fn update_logo_collection(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateLogoCollectionRequest>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let collection_id = path.into_inner();
    let user_id = claims.user_id;

    let result = crate::queries::logo_collections::update_logo_collection::update_logo_collection(
        pool.get_ref(),
        collection_id,
        user_id,
        payload.name.as_deref(),
        payload.description.as_ref().map(|s| Some(s.as_str())),
    )
    .await;

    match result {
        Ok(Some(collection)) => HttpResponse::Ok().json(collection),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Logo collection not found".to_string(),
        }),
        Err(err) => {
            eprintln!("Database error updating logo collection: {err:?}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update logo collection".to_string(),
            })
        }
    }
}
