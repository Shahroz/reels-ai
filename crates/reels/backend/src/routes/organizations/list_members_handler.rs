//! Handler for listing members of a specific organization.
// GET /api/organizations/{org_id}/members

use crate::auth::permissions::check_active_membership;
use crate::auth::tokens::Claims;
use crate::queries::organizations::list_members_for_organization;
use crate::routes::error_response::ErrorResponse;
use crate::routes::organizations::member_response::OrganizationMemberResponse;
use actix_web::{get, web, HttpResponse, Responder}; // Responder is already imported
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use utoipa::{self, ToSchema};

/// Response struct for listing organization members.
/// Wraps a vector of OrganizationMemberResponse structs, serializes transparently as an array.
#[derive(Serialize, ToSchema)]
#[serde(transparent)]
pub struct ListMembersResponse(pub Vec<OrganizationMemberResponse>);

#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/members",
    params(
        ("org_id" = Uuid, Path, description = "ID of the organization whose members to list")
    ),
    responses(
        (status = 200, description = "Successfully retrieved list of organization members", body = Vec<OrganizationMemberResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User is not an active member of the organization"),
        (status = 404, description = "Organization not found (though membership check should catch this first for members)"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[get("/{org_id}/members")]
pub async fn list_members_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> impl Responder {
    let org_id_to_list_members_for = path.into_inner();
    let user_id = claims.user_id;

    // 1. Check for active membership of the requesting user in the organization
    if let Err(response) =
        check_active_membership(pool.get_ref(), org_id_to_list_members_for, user_id).await
    {
        return response;
    }
    // If we reach here, the user is an active member.

    // 2. Fetch members for the organization
    match list_members_for_organization(pool.get_ref(), org_id_to_list_members_for).await {
        Ok(members) => {
            log::info!("Successfully fetched members for organization_id {org_id_to_list_members_for}: {members:?}");
            HttpResponse::Ok().json(ListMembersResponse(members))
        }
        Err(e) => {
            log::error!("Failed to list organization members for organization_id {org_id_to_list_members_for}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve organization members".to_string(),
            })
        }
    }
}