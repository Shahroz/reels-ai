//! Handler for updating an existing collection.
//!
//! PUT /api/collections/{id}

use crate::db::collections::Collection;
use crate::routes::collections::create_collection_request::CreateCollectionRequest;
use crate::routes::error_response::ErrorResponse;
use tracing::instrument;

#[utoipa::path(
    put,
    path = "/api/collections/{id}",
    tag = "Collections",
    request_body = CreateCollectionRequest,
    responses(
        (status = 200, description = "Updated", body = Collection),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::put("/{id}")]
#[instrument(skip(pool, payload))]
pub async fn update_collection(
    pool: actix_web::web::Data<sqlx::PgPool>,
    id: actix_web::web::Path<sqlx::types::Uuid>,
    payload: actix_web::web::Json<
        crate::routes::collections::create_collection_request::CreateCollectionRequest,
    >,
) -> impl actix_web::Responder {
    match crate::queries::collections::update_collection::update_collection(
        pool.get_ref(),
        *id,
        &payload.name,
        &payload.metadata,
        &payload.organization_id,
    )
    .await
    {
        Ok(item) => actix_web::HttpResponse::Ok().json(item),
        Err(sqlx::Error::RowNotFound) => {
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: "Collection not found".to_string(),
            })
        }
        Err(e) => actix_web::HttpResponse::InternalServerError().json(
            crate::routes::error_response::ErrorResponse {
                error: e.to_string(),
            },
        ),
    }
}
