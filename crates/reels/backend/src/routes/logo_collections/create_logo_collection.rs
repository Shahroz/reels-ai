//! Handler for creating a new logo collection.
//!
//! POST /api/logo-collections

use actix_web::web::{Data, Json, ReqData};
use actix_web::{HttpResponse};
use sqlx::types::Uuid;
use sqlx::PgPool;
use tracing::instrument;

use crate::db::logo_collection::LogoCollection;
use crate::schemas::logo_collection_schemas::CreateLogoCollectionRequest;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    post,
    path = "/api/logo-collections",
    tag = "Logo Collections",
    request_body = CreateLogoCollectionRequest,
    responses(
        (status = 201, description = "Created", body = LogoCollection),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::post("")]
#[instrument(skip(pool, payload, claims))]
pub async fn create_logo_collection(
    pool: Data<PgPool>,
    payload: Json<CreateLogoCollectionRequest>,
    claims: ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id: Uuid = claims.user_id;

    let result = crate::queries::logo_collections::create_logo_collection::create_logo_collection(
        pool.get_ref(),
        user_id,
        &payload.name,
        payload.description.as_deref(),
    )
    .await;

    match result {
        Ok(collection) => HttpResponse::Created().json(collection),
        Err(err) => {
            log::error!("Database error inserting logo collection for user {}: {:?}", user_id, err);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create logo collection".to_string(),
            })
        }
    }
}
