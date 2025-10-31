//! Handler for patching an asset's collection association.
//!
//! Defines the `patch_asset` HTTP handler under `/api/assets/{id}`.
//! This handler allows updating the collection_id of an asset.

use actix_web::{patch, web, HttpResponse, Responder};
use crate::db::assets::Asset;
use crate::routes::error_response::ErrorResponse;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(serde::Deserialize, Debug, ToSchema)]
pub struct PatchAssetRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub collection_id: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/assets/{id}",
    tag = "Assets",
    params(
        ("id" = String, Path, description = "Asset ID")
    ),
    request_body = PatchAssetRequest,
    responses(
        (status = 200, description = "Asset updated successfully", body = Asset),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Asset not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[patch("{id}")]
#[instrument(skip(pool, claims))]
pub async fn patch_asset(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<uuid::Uuid>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<PatchAssetRequest>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let asset_id = path.into_inner();
    let patch_data = req.into_inner();

    // Parse collection_id if provided
    let collection_id = patch_data.collection_id.as_ref().and_then(|id| {
        uuid::Uuid::parse_str(id).ok()
    });

    // First, verify the asset exists and belongs to the user
    let asset_result = crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, asset_id).await;

    match asset_result {
        Ok(Some(asset)) => {
            // Permission check: User must own the asset OR have editor access through organization shares
            let can_edit = if asset.user_id == Some(authenticated_user_id) {
                true
            } else {
                // Check if user has editor access through organization shares
                let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(&pool, authenticated_user_id).await {
                    Ok(memberships) => memberships,
                    Err(e) => {
                        log::error!("Error fetching org memberships (patch_asset) for user {authenticated_user_id}: {e}");
                        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check permissions."));
                    }
                };
                let org_ids: Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
                let org_ids_slice: &[uuid::Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };

                match sqlx::query_scalar!(
                    r#"
                        SELECT EXISTS (
                            SELECT 1 FROM object_shares
                            WHERE object_id = $1 AND object_type = $2 AND access_level = $3
                            AND (
                                (entity_type = 'user' AND entity_id = $4) OR
                                (entity_type = 'organization' AND entity_id = ANY($5))
                            )
                        )
                    "#,
                    asset_id, 
                    "asset", 
                    crate::db::shares::AccessLevel::Editor as crate::db::shares::AccessLevel,
                    authenticated_user_id,
                    org_ids_slice
                )
                .fetch_one(pool.as_ref())
                .await {
                    Ok(Some(true)) => true,
                    Ok(Some(false)) | Ok(None) => false,
                    Err(e) => {
                        log::error!("Error checking 'editor' share for asset patch {asset_id}: {e}");
                        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify permissions."));
                    }
                }
            };

            if can_edit {
                // Update the asset's collection
                let update_result = crate::queries::assets::update_asset_collection::update_asset_collection(
                    &pool,
                    asset_id,
                    collection_id,
                )
                .await;

                match update_result {
                    Ok(updated_asset) => HttpResponse::Ok().json(updated_asset),
                    Err(e) => {
                        log::error!(
                            "Error updating asset {asset_id} collection for user {authenticated_user_id}: {e}"
                        );
                        HttpResponse::InternalServerError().json(ErrorResponse::from(
                            "Failed to update asset collection"
                        ))
                    }
                }
            } else {
                HttpResponse::NotFound().json(ErrorResponse::from(
                    "Asset not found or access denied."
                ))
            }
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse::from("Asset not found.")),
        Err(e) => {
            log::error!(
                "Error retrieving asset {asset_id} for user {authenticated_user_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse::from(
                "Failed to retrieve asset"
            ))
        }
    }
} 