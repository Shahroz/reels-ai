//! Handler for fetching details of a specific organization.
// POST /api/organizations/{org_id} (this should be GET, correcting comment)
// GET /api/organizations/{org_id}

use crate::auth::permissions::check_active_membership;
use crate::auth::tokens::Claims;
use crate::db::organizations::Organization;
use crate::queries::organizations::find_organization_by_id;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use utoipa;

#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}",
    params(
        ("org_id" = Uuid, Path, description = "ID of the organization to fetch")
    ),
    responses(
        (status = 200, description = "Organization details fetched successfully", body = Organization),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User is not an active member of the organization"),
        (status = 404, description = "Organization not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[get("/{org_id}")]
pub async fn get_organization_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> impl Responder {
    let org_id_to_fetch = path.into_inner();
    let user_id = claims.user_id;

    // 1. Check for active membership
    if let Err(response) =
        check_active_membership(pool.get_ref(), org_id_to_fetch, user_id).await
    {
        return response;
    }
    // If we reach here, the user is an active member.

    // 2. Fetch organization details
    match find_organization_by_id(pool.get_ref(), org_id_to_fetch).await {
        Ok(Some(organization)) => HttpResponse::Ok().json(organization),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: format!("Organization not found: {org_id_to_fetch}"),
        }),
        Err(e) => {
            log::error!("Failed to fetch organization details: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve organization information".to_string(),
            })
        }
    }
}