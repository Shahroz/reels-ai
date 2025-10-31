//! Handler for transferring a custom creative format from an organization back to a user.
// POST /api/formats/{format_id}/transfer_from_organization

use crate::auth::tokens::Claims;
use crate::db::custom_creative_formats::{transfer_custom_creative_format_to_user, CustomCreativeFormat}; // Specific to formats
use crate::db::organization_members::{find_membership, OrganizationMemberStatus};
use crate::auth::permissions::check_active_owner;
use crate::queries::custom_creative_formats::get_organization_id::get_organization_id;
use crate::routes::error_response::ErrorResponse;
use crate::routes::objects_common::requests::TransferObjectFromOrganizationRequest;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use utoipa;
use log;

#[utoipa::path(
    post,
    path = "/api/formats/{format_id}/transfer_from_organization", // Path updated
    request_body = TransferObjectFromOrganizationRequest,
    params(
        ("format_id" = Uuid, Path, description = "ID of the custom creative format to transfer") // Param updated
    ),
    responses(
        (status = 200, description = "Custom Creative Format transferred successfully to user", body = CustomCreativeFormat), // Body updated
        (status = 400, description = "Bad Request (e.g., target user not a member, format not org-owned)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (e.g., user is not an owner of the format's organization) - Admin users can transfer any format"),
        (status = 404, description = "Custom Creative Format or Target User not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Formats" // Tag updated
)]
#[post("/{format_id}/transfer_from_organization")] // Route updated
pub async fn transfer_custom_creative_format_from_organization_handler(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    format_id_path: web::Path<Uuid>, // Param name updated
    payload: web::Json<TransferObjectFromOrganizationRequest>,
) -> impl Responder {
    let format_id_to_transfer = format_id_path.into_inner();
    let current_user_id = claims.user_id;
    let is_admin = claims.is_admin;
    let target_user_id = payload.target_user_id;

    let format_organization_id = match get_organization_id(pool.get_ref(), format_id_to_transfer).await {
        Ok(Some(Some(org_id))) => org_id,
        Ok(Some(None)) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Custom Creative Format is not currently owned by any organization.".to_string(),
            });
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Custom Creative Format not found.".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to fetch organization_id for custom creative format {}: {}", format_id_to_transfer, e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse { error: "Failed to retrieve custom creative format details.".to_string() });
        }
    };

    // Admin users can transfer any format, regular users can only transfer from organizations they own
    if !is_admin {
        if let Err(permission_err_response) =
            check_active_owner(pool.get_ref(), format_organization_id, current_user_id).await
        {
            return permission_err_response;
        }
    }

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            log::error!("Failed to begin transaction for member check: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database error while checking target user status.".to_string(),
            });
        }
    };

    match find_membership(&mut tx, format_organization_id, target_user_id).await {
        Ok(Some(membership)) => {
            if membership.status != OrganizationMemberStatus::Active.to_string() {
                let _ = tx.rollback().await;
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Target user is not an active member of the organization.".to_string(),
                });
            }
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction after member check: {}", e);
            }
        }
        Ok(None) => {
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Target user is not a member of the organization.".to_string(),
            });
        }
        Err(e) => {
            let _ = tx.rollback().await;
            log::error!("DB error checking target user membership for org {}: {}", format_organization_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify target user's membership status.".to_string(),
            });
        }
    }

    match transfer_custom_creative_format_to_user(pool.get_ref(), format_id_to_transfer, target_user_id).await { // Function call updated
        Ok(updated_format) => { // Variable name updated
            log::info!("Custom Creative Format {} successfully transferred from org {} to user {}", format_id_to_transfer, format_organization_id, target_user_id); // Log updated
            HttpResponse::Ok().json(updated_format)
        }
        Err(sqlx::Error::RowNotFound) => {
            log::warn!("Custom Creative Format {} not found during the final transfer update, though it was found earlier.", format_id_to_transfer); // Log updated
            HttpResponse::NotFound().json(ErrorResponse {
                error: "Custom Creative Format not found during transfer operation.".to_string(), // Text updated
            })
        }
        Err(e) => {
            log::error!("Failed to transfer custom creative format {} to user {}: {}", format_id_to_transfer, target_user_id, e); // Log updated
            HttpResponse::InternalServerError()
                .json(ErrorResponse { error: "Failed to update custom creative format ownership.".to_string() }) // Text updated
        }
    }
} 
