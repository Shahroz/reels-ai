//! Handler for removing an asset from a logo collection.
//!
//! DELETE /api/logo-collections/{id}/assets/{asset_id}

use actix_web::{delete, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::routes::error_response::ErrorResponse;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RemoveAssetFromLogoCollectionResponse {
    pub success: bool,
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/api/logo-collections/{id}/assets/{asset_id}",
    tag = "Logo Collections",
    params(
        ("id" = Uuid, Path, description = "Logo collection ID"),
        ("asset_id" = Uuid, Path, description = "Asset ID to remove")
    ),
    responses(
        (status = 200, description = "Success", body = RemoveAssetFromLogoCollectionResponse),
        (status = 404, description = "Collection or asset not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[delete("/{id}/assets/{asset_id}")]
#[instrument(skip(pool, claims))]
pub async fn remove_asset_from_logo_collection(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
) -> impl Responder {
    let (collection_id, asset_id) = path.into_inner();
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
            // Collection exists, proceed to remove asset
            let result = crate::queries::logo_collections::remove_asset_from_logo_collection::remove_asset_from_logo_collection(
                pool.get_ref(),
                collection_id,
                asset_id,
            )
            .await;

            match result {
                Ok(true) => HttpResponse::Ok().json(RemoveAssetFromLogoCollectionResponse {
                    success: true,
                    message: "Asset removed from logo collection successfully".to_string(),
                }),
                Ok(false) => HttpResponse::NotFound().json(ErrorResponse {
                    error: "Asset not found in this collection".to_string(),
                }),
                Err(err) => {
                    eprintln!("Database error removing asset from logo collection: {err:?}");
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to remove asset from logo collection".to_string(),
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
