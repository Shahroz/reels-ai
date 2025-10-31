//! Handler for attaching assets to a collection.
//!
//! Defines the `attach_assets` HTTP handler under `/api/assets/attach`.
//! This handler validates ownership and updates assets' collection_id fields in a transaction.
//! Supports both single asset and multiple asset operations through the same endpoint.
//! Follows the project's Rust coding standards with fully qualified paths.

use actix_web::{patch, web, HttpResponse, Responder};
use tracing::instrument;

use crate::auth::tokens::Claims;

#[derive(serde::Deserialize, utoipa::ToSchema, std::fmt::Debug)]
pub struct AttachAssetsRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub collection_id: uuid::Uuid,
    #[schema(example = "[\"550e8400-e29b-41d4-a716-446655440002\", \"550e8400-e29b-41d4-a716-446655440003\"]")]
    pub asset_ids: std::vec::Vec<uuid::Uuid>,
}

#[utoipa::path(
    patch,
    path = "/api/assets/attach",
    tag = "Assets",
    request_body = AttachAssetsRequest,
    responses(
        (status = 200, description = "Assets successfully attached to collection"),
        (status = 400, description = "Bad Request - Invalid input or empty asset list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden: User does not own one or more assets or the collection"),
        (status = 404, description = "Collection or one or more assets not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[patch("/attach")]
#[instrument(name = "attach_assets", skip(pool, claims))]
pub async fn attach_assets(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<AttachAssetsRequest>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    tracing::info!("Attaching assets to collection: collection_id={}, asset_ids={:?}, user_id={}", 
                   req.collection_id, req.asset_ids, claims.user_id);
    let user_id = claims.user_id;

    // Validate request
    if req.asset_ids.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Asset list cannot be empty"
        }));
    }

    // Enforce maximum operation limit for performance and security
    const MAX_ASSETS: usize = 100;
    if req.asset_ids.len() > MAX_ASSETS {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": std::format!("Cannot attach more than {} assets at once. Received {}", MAX_ASSETS, req.asset_ids.len())
        }));
    }

    // Start transaction for atomicity
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process attach request"
            }));
        }
    };

    // Debug: Log what we're trying to attach
    tracing::info!("Attempting to attach {} assets to collection {}", req.asset_ids.len(), req.collection_id);
    tracing::info!("Asset IDs: {:?}", req.asset_ids);

    // Validate collection and assets ownership using shared validation functions
    let _collection = match crate::routes::assets::validation::validate_collection_ownership::validate_collection_ownership(&mut *tx, req.collection_id, user_id).await {
        Ok(collection) => {
            tracing::info!("Collection {} found and owned by user {}", req.collection_id, user_id);
            collection
        },
        Err(response) => {
            tracing::error!("Collection validation failed for collection {}", req.collection_id);
            let _ = tx.rollback().await;
            return response;
        }
    };

    let _assets = match crate::routes::assets::validation::validate_bulk_asset_ownership::validate_bulk_asset_ownership(&mut *tx, &req.asset_ids, user_id).await {
        Ok(assets) => {
            tracing::info!("All {} assets found and owned by user {}", assets.len(), user_id);
            assets
        },
        Err(response) => {
            tracing::error!("Asset validation failed for assets: {:?}", req.asset_ids);
            let _ = tx.rollback().await;
            return response;
        }
    };

    // Perform atomic update
    let updated_count = match sqlx::query!(
        "UPDATE assets SET collection_id = $1, updated_at = NOW() WHERE id = ANY($2)",
        req.collection_id,
        &req.asset_ids
    )
    .execute(&mut *tx)
    .await {
        Ok(result) => result.rows_affected(),
        Err(e) => {
            tracing::error!("Database error during asset update: {:?}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to attach assets to collection"
            }));
        }
    };

    // Commit transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to complete attach operation"
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": std::format!("Successfully attached {} assets to collection", updated_count),
        "updated_count": updated_count
    }))
}