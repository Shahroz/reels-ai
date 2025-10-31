//! Handler for listing members of any organization (admin only).
//!
//! This endpoint allows administrators to view members of any organization,
//! bypassing the membership check required by the regular members endpoint.
//!
//! Revision History:
//! - 2025-10-15 @AI: Created to allow admins to view members without being in the org.

#[utoipa::path(
    get,
    path = "/api/admin/organizations/{organization_id}/members",
    tag = "Admin",
    params(
        ("organization_id" = uuid::Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved organization members", body = Vec<crate::routes::organizations::member_response::OrganizationMemberResponse>),
        (status = 403, description = "Forbidden - User is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/{organization_id}/members")]
#[tracing::instrument(skip(pool, auth_claims))]
pub async fn admin_list_members_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    organization_id: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    let org_id = organization_id.into_inner();

    // Verify organization exists
    match crate::queries::organizations::find_organization_by_id::find_organization_by_id(
        pool.get_ref(),
        org_id,
    )
    .await
    {
        Ok(Some(_)) => {
            // Organization exists, fetch members
            match crate::queries::organizations::list_members_for_organization::list_members_for_organization(
                pool.get_ref(),
                org_id,
            )
            .await
            {
                Ok(members) => {
                    log::info!(
                        "Admin {} successfully fetched {} members for organization {}",
                        auth_claims.user_id,
                        members.len(),
                        org_id
                    );
                    actix_web::HttpResponse::Ok().json(members)
                }
                Err(e) => {
                    log::error!(
                        "Admin {} failed to list members for organization {}: {}",
                        auth_claims.user_id,
                        org_id,
                        e
                    );
                    actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: String::from("Failed to retrieve organization members."),
                        },
                    )
                }
            }
        }
        Ok(None) => {
            log::warn!(
                "Admin {} tried to list members for non-existent organization: {}",
                auth_claims.user_id,
                org_id
            );
            actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: format!("Organization {} not found", org_id),
                },
            )
        }
        Err(e) => {
            log::error!(
                "Admin {} failed to verify organization {}: {}",
                auth_claims.user_id,
                org_id,
                e
            );
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: String::from("Failed to retrieve organization."),
                },
            )
        }
    }
}

