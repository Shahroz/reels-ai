//! Handler for adding an asset to a logo collection.
//!
//! POST /api/logo-collections/{id}/assets

use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::db::logo_collection_asset::LogoCollectionAsset;
use crate::schemas::logo_collection_schemas::AddAssetToCollectionRequest;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    post,
    path = "/api/logo-collections/{id}/assets",
    tag = "Logo Collections",
    params(
        ("id" = Uuid, Path, description = "Logo collection ID")
    ),
    request_body = AddAssetToCollectionRequest,
    responses(
        (status = 201, description = "Created", body = LogoCollectionAsset),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 409, description = "Asset already in collection", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[post("/{id}/assets")]
#[instrument(skip(pool, payload, claims))]
pub async fn add_asset_to_logo_collection(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    payload: web::Json<AddAssetToCollectionRequest>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let collection_id = path.into_inner();
    let user_id = claims.user_id;

    // First verify the collection exists and belongs to the user
    let collection_exists = crate::queries::logo_collections::get_logo_collection_by_id::get_logo_collection_by_id(
        pool.get_ref(),
        collection_id,
        user_id,
    )
    .await;

    match collection_exists {
        Ok(Some(_)) => {
            // Collection exists, proceed to add asset
            let result = crate::queries::logo_collections::add_asset_to_logo_collection::add_asset_to_logo_collection(
                pool.get_ref(),
                collection_id,
                payload.asset_id,
                payload.display_name.as_deref(),
            )
            .await;

            match result {
                Ok(collection_asset) => HttpResponse::Created().json(collection_asset),
                Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
                    HttpResponse::Conflict().json(ErrorResponse {
                        error: "Asset is already in this collection".to_string(),
                    })
                }
                Err(err) => {
                    eprintln!("Database error adding asset to logo collection: {err:?}");
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to add asset to logo collection".to_string(),
                    })
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Logo collection not found".to_string(),
        }),
        Err(err) => {
            eprintln!("Database error checking logo collection: {err:?}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify logo collection".to_string(),
            })
        }
    }
}
