// Handler for rejecting an organization invitation.
// POST /api/invitations/reject

use crate::auth::invitation_tokens::validate_invitation_token;
use crate::db::organization_members::{OrganizationMemberStatus, OrganizationMember};
use crate::queries::organizations::{find_membership, update_member_status_and_role};
use crate::db::users::find_user_by_email;
use crate::routes::error_response::ErrorResponse;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use crate::auth::tokens::get_jwt_secret;
use log;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectInvitationRequest {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/invitations/reject",
    request_body = RejectInvitationRequest,
    responses(
        (status = 200, description = "Invitation rejected successfully.", body = OrganizationMember), // Or 204 No Content
        (status = 400, description = "Bad Request (e.g., invalid or expired token)"),
        (status = 404, description = "User or Invitation not found based on token data"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    tag = "Invitations"
)]
#[post("/reject")]
pub async fn reject_invitation_handler(
    pool: web::Data<PgPool>,
    payload: web::Json<RejectInvitationRequest>,
) -> impl Responder {
    let issuer = "narrativ.com"; // TODO: from config
    let audience = "narrativ_invitation"; // TODO: from config
    
    let jwt_secret = match get_jwt_secret() {
        Ok(secret) => secret,
        Err(e) => {
            log::error!("JWT_SECRET not configured for invitation rejection: {e}. Cannot validate token.");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Server configuration error preventing token validation.".to_string(),
            });
        }
    };

    let invitation_claims = match validate_invitation_token(&payload.token, issuer, audience, &jwt_secret) {
        Ok(claims) => claims,
        Err(e) => {
            log::warn!("Invalid invitation token for rejection: {}. Error: {}", payload.token, e);
            let err_msg = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Invitation token has expired.",
                _ => "Invalid or malformed invitation token.",
            };
            return HttpResponse::BadRequest().json(ErrorResponse { error: err_msg.to_string() });
        }
    };

    let user_to_reject = match find_user_by_email(pool.get_ref(), &invitation_claims.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("User with email {} from token not found for org {} during rejection.", invitation_claims.email, invitation_claims.org_id);
            return HttpResponse::NotFound().json(ErrorResponse { error: "User specified in invitation not found.".to_string() });
        }
        Err(e) => {
            log::error!("DB error finding user by email {}: {}", invitation_claims.email, e);
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Error verifying user from invitation.".to_string() });
        }
    };

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            log::error!("Failed to begin transaction for rejecting invitation: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error initiating reject invitation.".to_string() });
        }
    };

    match find_membership(&mut tx, invitation_claims.org_id, user_to_reject.id).await {
        Ok(Some(membership)) => {
            if membership.status != OrganizationMemberStatus::Invited.to_string() {
                let _ = tx.rollback().await;
                log::warn!("Invitation for user {} to org {} has status '{}', not 'invited'. Cannot reject.", user_to_reject.id, invitation_claims.org_id, membership.status);
                // Consider if already active/rejected should be a specific conflict or bad request.
                return HttpResponse::BadRequest().json(ErrorResponse { error: "This invitation cannot be rejected (it may have already been processed or is in an invalid state).".to_string() });
            }

            // Update status to 'rejected'. Role is not relevant for rejection, so pass None.
            match update_member_status_and_role(&mut tx, invitation_claims.org_id, user_to_reject.id, OrganizationMemberStatus::Rejected, None).await {
                Ok(updated_member) => {
                    if let Err(e) = tx.commit().await {
                        log::error!("Failed to commit transaction for rejecting invitation: {e}");
                        return HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to finalize invitation rejection.".to_string() });
                    }
                    // Decide on response: 200 with body or 204 No Content.
                    // Returning body for consistency with accept, for now.
                    HttpResponse::Ok().json(updated_member)
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    log::error!("Failed to update member status to rejected for user {} in org {}: {}", user_to_reject.id, invitation_claims.org_id, e);
                    HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to update membership status for rejection.".to_string() })
                }
            }
        }
        Ok(None) => {
            let _ = tx.rollback().await;
            log::error!("No pending invitation found for user {} (email {}) to organization {} for rejection.", user_to_reject.id, invitation_claims.email, invitation_claims.org_id);
            HttpResponse::NotFound().json(ErrorResponse { error: "No matching invitation found to reject.".to_string() })
        }
        Err(e) => {
            let _ = tx.rollback().await;
            log::error!("DB error checking existing membership for user {} in org {} for rejection: {}", user_to_reject.id, invitation_claims.org_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "Error verifying invitation details for rejection.".to_string() })
        }
    }
} 