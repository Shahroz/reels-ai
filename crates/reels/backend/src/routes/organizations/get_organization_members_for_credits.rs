//! Route handler to get organization members for credit filtering.
//!
//! This endpoint returns the list of members for an organization,
//! used to populate user filter dropdowns in credit usage dashboards.
//! Requires the requesting user to be a member of the organization.

#[utoipa::path(
    get,
    path = "/api/organizations/{organization_id}/members",
    tag = "Organizations",
    params(
        ("organization_id" = uuid::Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "List of organization members", body = Vec<crate::routes::organizations::member_response::OrganizationMemberResponse>),
        (status = 403, description = "User is not a member of this organization", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse),
    ),
    security(
        ("bearer" = [])
    )
)]
#[actix_web::get("/organizations/{organization_id}/members")]
pub async fn get_organization_members_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    path: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let organization_id = path.into_inner();
    
    log::info!(
        "User {} requesting members for organization {}",
        user_id,
        organization_id
    );
    
    // Verify user is a member of the organization
    match crate::queries::organizations::verify_organization_membership(pool.get_ref(), user_id, organization_id).await {
        Ok(true) => {
            log::info!("User {} is a member of organization {}", user_id, organization_id);
        },
        Ok(false) => {
            log::warn!(
                "User {} attempted to access members of organization {} but is not a member",
                user_id,
                organization_id
            );
            return actix_web::HttpResponse::Forbidden().json(crate::routes::error_response::ErrorResponse {
                error: "You are not a member of this organization".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to verify organization membership: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to verify organization membership".to_string(),
            });
        }
    }
    
    // Fetch organization members using existing query
    match crate::queries::organizations::list_members_for_organization(pool.get_ref(), organization_id).await {
        Ok(members) => {
            log::info!(
                "Retrieved {} members for organization {}",
                members.len(),
                organization_id
            );
            actix_web::HttpResponse::Ok().json(members)
        },
        Err(e) => {
            log::error!("Failed to fetch organization members: {}", e);
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to fetch organization members".to_string(),
            })
        }
    }
}

