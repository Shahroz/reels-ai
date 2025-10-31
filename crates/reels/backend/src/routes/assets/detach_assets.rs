//! Handler for detaching assets from their collections.
//!
//! Defines the `detach_assets` HTTP handler under `/api/assets/detach`.
//! This handler validates ownership and removes assets' collection_id associations.
//! Supports both single asset and multiple asset operations through the same endpoint.
//! Follows the project's Rust coding standards with fully qualified paths.

use actix_web::{patch, web, HttpResponse, Responder};
use tracing::instrument;

use crate::auth::tokens::Claims;

#[derive(serde::Deserialize, utoipa::ToSchema, std::fmt::Debug)]
pub struct DetachAssetsRequest {
    #[schema(example = "[\"550e8400-e29b-41d4-a716-446655440002\", \"550e8400-e29b-41d4-a716-446655440003\"]")]
    pub asset_ids: std::vec::Vec<uuid::Uuid>,
}

#[utoipa::path(
    patch,
    path = "/api/assets/detach",
    tag = "Assets",
    request_body = DetachAssetsRequest,
    responses(
        (status = 200, description = "Assets successfully detached from collections"),
        (status = 400, description = "Bad Request - Invalid input or empty asset list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden: User does not own one or more assets"),
        (status = 404, description = "One or more assets not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[patch("/detach")]
#[instrument(name = "detach_assets", skip(pool, claims))]
pub async fn detach_assets(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<DetachAssetsRequest>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
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
            "error": std::format!("Cannot detach more than {} assets at once. Received {}", MAX_ASSETS, req.asset_ids.len())
        }));
    }

    // Start transaction for atomicity
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process detach request"
            }));
        }
    };

    // Validate assets ownership using shared validation function
    let _assets = match crate::routes::assets::validation::validate_bulk_asset_ownership::validate_bulk_asset_ownership(&mut *tx, &req.asset_ids, user_id).await {
        Ok(assets) => assets,
        Err(response) => {
            let _ = tx.rollback().await;
            return response;
        }
    };

    // Perform atomic update (set collection_id to NULL)
    let updated_count = match sqlx::query!(
        "UPDATE assets SET collection_id = NULL, updated_at = NOW() WHERE id = ANY($1)",
        &req.asset_ids
    )
    .execute(&mut *tx)
    .await {
        Ok(result) => result.rows_affected(),
        Err(e) => {
            tracing::error!("Database error during asset detach: {:?}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to detach assets from collections"
            }));
        }
    };

    // Commit transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to complete detach operation"
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": std::format!("Successfully detached {} assets from collections", updated_count),
        "updated_count": updated_count
    }))
}