//! Handler for deleting an organization.
// DELETE /api/organizations/{org_id}

use crate::auth::permissions::check_active_owner;
use crate::auth::tokens::Claims;
use crate::queries::organizations::{delete_organization_by_id, find_organization_by_id};
use crate::routes::error_response::ErrorResponse;
use actix_web::{delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use utoipa;

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}",
    params(
        ("org_id" = Uuid, Path, description = "ID of the organization to delete")
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User is not an owner or organization still owns objects"),
        (status = 404, description = "Organization not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[delete("/{org_id}")]
pub async fn delete_organization_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> impl Responder {
    let org_id_to_delete = path.into_inner();
    let user_id = claims.user_id;

    // 0. Check if organization exists first
    let organization = match find_organization_by_id(pool.get_ref(), org_id_to_delete).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Organization not found: {org_id_to_delete}"),
            });
        }
        Err(e) => {
            log::error!("DB error checking org existence for {org_id_to_delete}: {e}. Returning 500");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to verify organization existence".to_string(),
            });
        }
    };

    // 0.5. Check if organization is a personal organization
    if organization.is_personal {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Personal organizations cannot be deleted. They are automatically created for each user and are required.".to_string(),
        });
    }

    // 1. Permission Check: User must be an active owner
    if let Err(response) = check_active_owner(pool.get_ref(), org_id_to_delete, user_id).await {
        return response;
    }

    // 2. Delete the organization
    match delete_organization_by_id(pool.get_ref(), org_id_to_delete).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                HttpResponse::NoContent().finish() // 204 No Content
            } else {
                // This case should ideally not be hit if the existence check passed, but is kept for safety.
                HttpResponse::NotFound().json(ErrorResponse {
                    error: format!("Organization not found during deletion attempt: {org_id_to_delete}"),
                })
            }
        }
        Err(e) => {
            log::error!("DB error deleting org {org_id_to_delete}: {e}. Returning 500");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete organization".to_string(),
            })
        }
    }
}