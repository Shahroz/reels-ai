use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::db::assets::Asset; // Adjusted path
use crate::queries::organizations::find_membership;
use crate::routes::error_response::ErrorResponse;

#[derive(Deserialize, Debug, ToSchema)]
pub struct ShareToOrganizationRequest {
    #[schema(example = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")]
    pub organization_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/api/assets/{asset_id}/share_to_organization", // Adjusted path and ID
    tag = "Assets", // Adjusted tag
    security(("bearer_auth" = [])),
    params(
        ("asset_id" = Uuid, Path, description = "ID of the asset to share") // Adjusted ID
    ),
    request_body = ShareToOrganizationRequest,
    responses(
        (status = 200, description = "Asset shared successfully with the organization", body = Asset), // Adjusted body
        (status = 400, description = "Bad Request (e.g., asset already shared)", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (not owner, or not active member of org)", body = ErrorResponse),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn share_asset_to_organization(
    pool: web::Data<PgPool>,
    asset_id: web::Path<Uuid>, // Adjusted ID
    claims: web::ReqData<Claims>,
    payload: web::Json<ShareToOrganizationRequest>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let a_id = *asset_id; // Adjusted ID
    let target_organization_id = payload.organization_id;

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction for sharing asset: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to start transaction."));
        }
    };

    // 1. Fetch the asset. Fields must match the Asset struct.
    // Assuming Asset struct includes: id, user_id, organization_id, name, asset_type, file_path, file_type, file_size, created_at, updated_at
    let asset = match sqlx::query_as!(
        Asset,
        "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, created_at, updated_at, is_public FROM assets WHERE id = $1 FOR UPDATE",
        a_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            return HttpResponse::NotFound().json(ErrorResponse::from("Asset not found."));
        }
        Err(e) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            log::error!("Failed to fetch asset {} for sharing: {}", a_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve asset."));
        }
    };

    // 2. Check 1: Asset ownership and current state
    if asset.organization_id.is_some() {
        if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
        return HttpResponse::BadRequest().json(ErrorResponse::from("Asset is already owned by an organization."));
    }

    // Ensure user owns the asset (public assets with user_id = None cannot be shared)
    if asset.user_id != Some(authenticated_user_id) {
        if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
        return HttpResponse::Forbidden().json(ErrorResponse::from("You do not own this asset."));
    }

    // 3. Check 2: User's membership in the target organization
    // Use a separate transaction for membership check to avoid type conflicts
    let mut tx_for_membership = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction for membership check: {}", e);
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to start membership check transaction."));
        }
    };
    
    match find_membership(&mut tx_for_membership, target_organization_id, authenticated_user_id).await {
        Ok(Some(membership)) => {
            if membership.status.to_lowercase() != "active" {
                if let Err(e_rb) = tx_for_membership.rollback().await { log::error!("Rollback failed: {}", e_rb); }
                if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
                return HttpResponse::Forbidden().json(ErrorResponse::from("You are not an active member of the target organization."));
            }
        }
        Ok(None) => {
            if let Err(e_rb) = tx_for_membership.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            return HttpResponse::Forbidden().json(ErrorResponse::from("You are not a member of the target organization."));
        }
        Err(e) => {
            if let Err(e_rb) = tx_for_membership.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            log::error!("Failed to verify organization membership for user {} in org {}: {}", authenticated_user_id, target_organization_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify organization membership."));
        }
    }
    
    // Commit the membership check transaction
    if let Err(e) = tx_for_membership.commit().await {
        log::error!("Failed to commit membership check transaction: {}", e);
        if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify organization membership."));
    }

    // 4. Perform the update
    let updated_asset = match sqlx::query_as!(
        Asset,
        "UPDATE assets SET updated_at = NOW() WHERE id = $1 RETURNING id, user_id, name, type, gcs_object_name, url, collection_id, created_at, updated_at, is_public",
        a_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(a) => a,
        Err(e) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            log::error!("Failed to share asset {} to organization {}: {}", a_id, target_organization_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to share asset."));
        }
    };

    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction for sharing asset: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize sharing asset."));
    }

    HttpResponse::Ok().json(updated_asset)
} 