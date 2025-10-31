//! Handler for removing a member from an organization or a user leaving an organization.
// DELETE /api/organizations/{org_id}/members/{user_id_to_remove}

use crate::auth::permissions::check_active_membership;
use crate::auth::tokens::Claims;
use crate::queries::organizations::{find_membership, remove_member};
use crate::routes::error_response::ErrorResponse;
use actix_web::{delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use utoipa;
use log::error;

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/members/{user_id_to_remove}",
    params(
        ("org_id" = Uuid, Path, description = "ID of the organization"),
        ("user_id_to_remove" = Uuid, Path, description = "ID of the user to remove or for self-removal")
    ),
    responses(
        (status = 204, description = "Member removed or user left successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Insufficient permissions or invalid operation"),
        (status = 404, description = "Organization or member not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[delete("/{org_id}/members/{user_id_to_remove}")]
pub async fn remove_member_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (org_id, user_id_to_remove) = path.into_inner();
    let authenticated_user_id = claims.user_id;

    // Check if the authenticated user is an active member of the organization
    let auth_user_member = match check_active_membership(pool.get_ref(), org_id, authenticated_user_id).await {
        Ok(member) => member,
        Err(response) => return response,
    };

    // Scenario 1: Self-removal (leaving organization)
    if authenticated_user_id == user_id_to_remove {
        if auth_user_member.role == "owner" {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Owners cannot leave their organization. Please delete the organization or transfer ownership (feature not available in Stage 1)."
                    .to_string(),
            });
        }
        // If member is not owner, they can leave.
    } else {
        // Scenario 2: Removing another user
        if auth_user_member.role != "owner" {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Access denied: Only an owner can remove other members.".to_string(),
            });
        }

        // Fetch target user's membership to ensure they are not an owner
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to begin transaction: {e}");
                return HttpResponse::InternalServerError().json(ErrorResponse{ error: "Database transaction failed.".to_string() });
            }
        };

        let target_user_membership_opt = match find_membership(&mut tx, org_id, user_id_to_remove).await {
            Ok(opt_member) => opt_member,
            Err(e) => {
                error!("Failed to query organization membership: {e}");
                let _ = tx.rollback().await; // Attempt to rollback
                return HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to query organization membership".to_string() });
            }
        };

        if target_user_membership_opt.is_none() {
            let _ = tx.rollback().await; // Attempt to rollback
            return HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Target user {user_id_to_remove} is not a member of organization {org_id}."),
            });
        }
        
        let _ = tx.commit().await; // Commit transaction if checks passed

        let target_member = target_user_membership_opt.unwrap();
        if target_member.role == "owner" {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Cannot remove an owner from the organization.".to_string(),
            });
        }
    }

    // Proceed with removal
    match remove_member(pool.get_ref(), org_id, user_id_to_remove).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                HttpResponse::NoContent().finish() // 204 No Content
            } else {
                // This case should ideally be caught by the checks above, but as a fallback:
                HttpResponse::NotFound().json(ErrorResponse {
                    error: format!("Member {user_id_to_remove} not found in organization {org_id}."),
                })
            }
        }
        Err(e) => {
            log::error!("Failed to remove member: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to remove member from organization.".to_string(),
            })
        }
    }
} 
