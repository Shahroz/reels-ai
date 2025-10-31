//! Handler for updating an existing organization.
// PUT /api/organizations/{org_id}

use crate::auth::permissions::check_active_owner;
use crate::auth::tokens::Claims;
use crate::db::organizations::Organization;
use crate::queries::organizations::update_organization_details;
use crate::routes::error_response::ErrorResponse;
use crate::routes::organizations::update_organization_request::UpdateOrganizationRequest;
use actix_web::{put, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use utoipa;

#[utoipa::path(
    put,
    path = "/api/organizations/{org_id}",
    request_body = UpdateOrganizationRequest,
    params(
        ("org_id" = Uuid, Path, description = "ID of the organization to update")
    ),
    responses(
        (status = 200, description = "Organization updated successfully", body = Organization),
        (status = 400, description = "Bad request (e.g., no update fields provided)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User is not an owner of the organization"),
        (status = 404, description = "Organization not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[put("/{org_id}")]
pub async fn update_organization_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateOrganizationRequest>,
) -> impl Responder {
    let org_id_to_update = path.into_inner();
    let user_id = claims.user_id;

    // 1. Permission Check: User must be an active owner of the organization
    if let Err(response) =
        check_active_owner(pool.get_ref(), org_id_to_update, user_id).await
    {
        return response;
    }
    // If we reach here, the user is an active owner.

    // Prevent attempting an update if no actual data is provided in the payload
    if payload.name.is_none() /* && other_fields.is_none() ... */ {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No update data provided. At least one field (e.g., name) must be specified.".to_string(),
        });
    }

    // 2. Update organization details
    match update_organization_details(pool.get_ref(), org_id_to_update, payload.name.clone()).await {
        Ok(Some(updated_organization)) => HttpResponse::Ok().json(updated_organization),
        Ok(None) => {
            // This case implies find_organization_by_id was called by update_organization_details
            // because payload.name was None, or the organization was not found after an attempted update.
            // If update_organization_details only attempts update when name is Some, then Ok(None) means not found.
            HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Organization not found or no update performed: {org_id_to_update}"),
            })
        }
        Err(e) => {
            log::error!("Failed to update organization: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update organization information".to_string(),
            })
        }
    }
}