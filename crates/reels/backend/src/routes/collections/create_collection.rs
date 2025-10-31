//! Handler for creating a new collection.
//!
//! POST /api/collections

use actix_web::web::{Data, Json, ReqData};
use actix_web::{HttpResponse};
use sqlx::types::Uuid;
use sqlx::PgPool;
use tracing::instrument;

use crate::db::collections::Collection;
use crate::routes::error_response::ErrorResponse;
use crate::routes::collections::create_collection_request::CreateCollectionRequest;

#[utoipa::path(
    post,
    path = "/api/collections",
    tag = "Collections",
    request_body = CreateCollectionRequest,
    responses(
        (status = 201, description = "Created", body = Collection),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::post("")]
#[instrument(skip(pool, payload, claims, req))]
pub async fn create_collection(
    pool: Data<PgPool>,
    payload: Json<crate::routes::collections::create_collection_request::CreateCollectionRequest>,
    claims: ReqData<crate::auth::tokens::Claims>,
    req: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let user_id: Uuid = claims.user_id;

    // Get organization_id from request payload or header (payload takes precedence)
    let organization_id = payload.organization_id
        .or_else(|| crate::services::credits_service::extract_organization_id_from_headers(&req));

    let result = crate::queries::collections::create_collection::create_collection(
        pool.get_ref(),
        user_id,
        &payload.name,
        &payload.metadata,
        &organization_id,
    )
    .await;

    match result {
        Ok(collection) => HttpResponse::Created().json(collection),
        Err(err) => {
            eprintln!("Database error inserting collection: {err:?}");
            HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to create collection".to_string(),
            })
        }
    }
}
