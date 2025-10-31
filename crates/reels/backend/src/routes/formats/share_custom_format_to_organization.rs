use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::db::custom_creative_formats::CustomCreativeFormat; // Adjusted path
use crate::db::organization_members;
use crate::queries::custom_creative_formats::{find_for_update, share_with_org};
use crate::routes::error_response::ErrorResponse;

#[derive(Deserialize, Debug, ToSchema)]
pub struct ShareToOrganizationRequest {
    #[schema(example = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")]
    pub organization_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/api/formats/custom-creative-formats/{format_id}/share_to_organization", // Adjusted path and ID
    tag = "Formats", // Tag remains Formats (covers custom formats)
    security(("bearer_auth" = [])),
    params(
        ("format_id" = Uuid, Path, description = "ID of the custom creative format to share") // Adjusted ID
    ),
    request_body = ShareToOrganizationRequest,
    responses(
        (status = 200, description = "Custom creative format shared successfully with the organization", body = CustomCreativeFormat), // Adjusted body
        (status = 400, description = "Bad Request (e.g., format already shared)", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (not owner, or not active member of org) - Admin users can share any format", body = ErrorResponse),
        (status = 404, description = "Custom creative format not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
pub async fn share_custom_format_to_organization(
    pool: web::Data<PgPool>,
    format_id: web::Path<Uuid>, // Adjusted ID
    claims: web::ReqData<Claims>,
    payload: web::Json<ShareToOrganizationRequest>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let is_admin = claims.is_admin;
    let f_id = *format_id; // Adjusted ID
    let target_organization_id = payload.organization_id;

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to begin transaction for sharing custom format: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to start transaction."));
        }
    };

    // 1. Fetch the custom creative format, locking it for update.
    let custom_format = match find_for_update::find_for_update(&mut *tx, f_id).await {
        Ok(Some(cf)) => cf,
        Ok(None) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            return HttpResponse::NotFound().json(ErrorResponse::from("Custom creative format not found."));
        }
        Err(e) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            log::error!("Failed to fetch custom format {} for sharing: {}", f_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve custom format."));
        }
    };

    // 2. Check 1: Custom format ownership and current state
    if custom_format.organization_id.is_some() {
        if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
        return HttpResponse::BadRequest().json(ErrorResponse::from("Custom creative format is already owned by an organization."));
    }

    // Admin users can share any format, regular users can only share their own
    if !is_admin && custom_format.user_id != Some(authenticated_user_id) {
        if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
        return HttpResponse::Forbidden().json(ErrorResponse::from("You do not own this custom creative format."));
    }

    // 3. Check 2: User's membership in the target organization
    match organization_members::find_membership(&mut *tx, target_organization_id, authenticated_user_id).await {
        Ok(Some(membership)) => {
            if membership.status.to_lowercase() != "active" {
                if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
                return HttpResponse::Forbidden().json(ErrorResponse::from("You are not an active member of the target organization."));
            }
        }
        Ok(None) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            return HttpResponse::Forbidden().json(ErrorResponse::from("You are not a member of the target organization."));
        }
        Err(e) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            log::error!("Failed to verify organization membership for user {} in org {}: {}", authenticated_user_id, target_organization_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify organization membership."));
        }
    }

    // 4. Perform the update
    let updated_custom_format =
        match share_with_org::share_with_org(&mut *tx, f_id, target_organization_id).await {
        Ok(cf) => cf,
        Err(e) => {
            if let Err(e_rb) = tx.rollback().await { log::error!("Rollback failed: {}", e_rb); }
            log::error!("Failed to share custom format {} to organization {}: {}", f_id, target_organization_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to share custom creative format."));
        }
    };

    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction for sharing custom format: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize sharing custom format."));
    }

    HttpResponse::Ok().json(updated_custom_format)
} 
