//! Handler for retrieving vocal tour assets by document ID.
//!
//! This endpoint allows fetching the assets associated with a vocal tour document.
//! It looks up the vocal tour by document ID and returns the associated assets.
//! This is useful for the frontend to get actual asset IDs for AI Studio integration.

use actix_web::{get, web, HttpResponse, Responder};
use crate::routes::assets::error_response::ErrorResponse;
use tracing::instrument;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct VocalTourAssetsResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub vocal_tour_id: uuid::Uuid,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub document_id: uuid::Uuid,
    pub assets: std::vec::Vec<crate::db::assets::Asset>,
}

#[utoipa::path(
    get,
    path = "/api/vocal-tour/assets/{document_id}",
    tag = "Vocal Tour",
    params(
        ("document_id" = String, Path, description = "Document ID to get vocal tour assets for", example = "550e8400-e29b-41d4-a716-446655440000")
    ),
    responses(
        (status = 200, description = "Vocal tour assets retrieved successfully", body = VocalTourAssetsResponse),
        (status = 400, description = "Bad Request - Invalid document ID format"),
        (status = 404, description = "Vocal tour not found for this document"),
        (status = 500, description = "Internal Server Error - Database error")
    ),
    security(("user_auth" = []))
)]
#[get("/assets/{document_id}")]
#[instrument(skip(pool, claims))]
pub async fn get_vocal_tour_assets(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    path: web::Path<String>,
) -> impl Responder {
    let user_id = claims.user_id;
    let document_id_str = path.into_inner();

    // Parse document ID
    let document_id = match uuid::Uuid::parse_str(&document_id_str) {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(e) => {
            log::warn!("Invalid document ID format '{document_id_str}' for user {user_id}: {e}");
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid document ID format: {document_id_str}"),
            });
        }
    };

    // Get vocal tour by document ID
    let vocal_tour = match crate::queries::vocal_tours::get_vocal_tour_by_document_id::get_vocal_tour_by_document_id(&pool, document_id).await {
        std::result::Result::Ok(std::option::Option::Some(vocal_tour)) => {
            // Verify user owns this vocal tour
            if vocal_tour.user_id != user_id {
                log::warn!("User {user_id} attempted to access vocal tour for document {document_id} owned by another user");
                return HttpResponse::NotFound().json(ErrorResponse {
                    error: "Vocal tour not found for this document".into(),
                });
            }
            vocal_tour
        }
        std::result::Result::Ok(std::option::Option::None) => {
            log::info!("No vocal tour found for document {document_id} for user {user_id}");
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Vocal tour not found for this document".into(),
            });
        }
        std::result::Result::Err(e) => {
            log::error!("Database error fetching vocal tour for document {document_id} for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch vocal tour".into(),
            });
        }
    };

    // Fetch all assets associated with this vocal tour
    let mut assets = std::vec::Vec::new();
    let vocal_tour_id = vocal_tour.id;
    for asset_id in &vocal_tour.asset_ids {
        match crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, *asset_id).await {
            std::result::Result::Ok(std::option::Option::Some(asset)) => {
                // Double-check user ownership
                if asset.user_id == Some(user_id) {
                    assets.push(asset);
                } else {
                    log::warn!("Asset {} in vocal tour {} is not owned by user {}", asset_id, vocal_tour_id, user_id);
                }
            }
            std::result::Result::Ok(std::option::Option::None) => {
                log::warn!("Asset {} referenced in vocal tour {} not found", asset_id, vocal_tour_id);
            }
            std::result::Result::Err(e) => {
                log::error!("Database error fetching asset {} for vocal tour {}: {}", asset_id, vocal_tour_id, e);
            }
        }
    }

    log::info!("Retrieved {} assets for vocal tour {} (document {}) for user {}", 
               assets.len(), vocal_tour_id, document_id, user_id);

    HttpResponse::Ok().json(VocalTourAssetsResponse {
        vocal_tour_id: vocal_tour.id,
        document_id,
        assets,
    })
}
