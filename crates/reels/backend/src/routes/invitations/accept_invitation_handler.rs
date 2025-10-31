// Handler for accepting an organization invitation.
// POST /api/invitations/accept

use crate::auth::tokens::Claims; // Added: Requires authentication
use crate::db::organization_members::{OrganizationMember, OrganizationMemberStatus}; // Corrected based on usage
use crate::queries::organizations::add_member; // Corrected based on usage
use crate::db::users::find_user_by_id; // Removed unused: find_user_by_email, create_user as db_create_user, User as DbUser
use crate::routes::error_response::ErrorResponse;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool; // Removed unused: Transaction, Postgres
use utoipa::ToSchema;
use log;
use crate::queries::pending_invitations::delete_pending_invitation::delete_pending_invitation;
use crate::queries::pending_invitations::find_pending_invitation_by_token::find_pending_invitation_by_token;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInvitationRequest {
    #[schema(example = "actual_token_string_from_pending_invitations_table")]
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/invitations/accept",
    request_body = AcceptInvitationRequest,
    responses(
        (status = 200, description = "Invitation accepted successfully.", body = OrganizationMember),
        (status = 400, description = "Bad Request (e.g., invalid or expired token)", body = ErrorResponse),
        (status = 401, description = "Unauthorized (user not authenticated or user data mismatch)", body = ErrorResponse),
        (status = 403, description = "Forbidden (invitation is for a different user)", body = ErrorResponse),
        (status = 404, description = "Invitation not found or user details not found for authenticated user", body = ErrorResponse),
        (status = 409, description = "Conflict (e.g., already an active member)", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security( // Added: This endpoint now requires authentication
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
#[post("/accept")]
pub async fn accept_invitation_handler(
    pool: web::Data<PgPool>,
    payload: web::Json<AcceptInvitationRequest>,
    claims: Claims, // Authenticated user claims (user_id, exp)
) -> impl Responder {
    let token_str = &payload.token;

    // 1. Find the pending invitation by the raw token string.
    let pending_invite = match find_pending_invitation_by_token(pool.get_ref(), token_str).await {
        Ok(Some(invite)) => invite,
        Ok(None) => {
            log::warn!("Accept attempt with non-existent token: {token_str}");
            return HttpResponse::NotFound().json(ErrorResponse { error: "Invitation not found or already used.".to_string() });
        }
        Err(e) => {
            log::error!("DB error finding pending invitation by token {token_str}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Error verifying invitation details.".to_string() });
        }
    };

    // 2. Check if the token has expired.
    if pending_invite.token_expires_at < Utc::now() {
        log::warn!("Attempt to accept expired invitation token ID: {}. Expires_at: {}, Now: {}", pending_invite.id, pending_invite.token_expires_at, Utc::now());
        // Acquire a connection to delete the expired token
        match pool.acquire().await {
            Ok(mut conn) => {
                if let Err(e) = delete_pending_invitation(&mut conn, pending_invite.id).await {
                    log::error!("Failed to delete expired pending invitation {}: {}", pending_invite.id, e);
                    // Not returning an error to client, as this is a cleanup task
                }
            }
            Err(e) => {
                log::error!("Failed to acquire DB connection to delete expired invitation {}: {}", pending_invite.id, e);
                // Not returning an error to client, as this is a cleanup task
            }
        }
        return HttpResponse::BadRequest().json(ErrorResponse { error: "Invitation token has expired.".to_string() });
    }

    // 3. Fetch authenticated user's details to get their email.
    let authenticated_user = match find_user_by_id(pool.get_ref(), claims.user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!(
                "Authenticated user_id {} not found in DB while trying to accept invitation ID {}. This should not happen.",
                claims.user_id, pending_invite.id
            );
            return HttpResponse::NotFound().json(ErrorResponse { error: "Authenticated user data not found.".to_string() });
        }
        Err(e) => {
            log::error!("DB error fetching authenticated user {} details: {}", claims.user_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Error fetching user details.".to_string() });
        }
    };

    // 4. Verify that the authenticated user's email matches the invited email.
    if authenticated_user.email.to_lowercase() != pending_invite.invited_email.to_lowercase() {
        log::warn!(
            "User {} (email {}) attempted to accept invitation ID {} intended for email {}.",
            claims.user_id, authenticated_user.email, pending_invite.id, pending_invite.invited_email
        );
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: format!(
                "This invitation is for {}. You are logged in as {}. Please log in with the correct account.",
                pending_invite.invited_email, authenticated_user.email
            ),
        });
    }

    // User is authenticated as the correct invitee. Proceed with transaction.
    let user_id_to_add = claims.user_id; // This is authenticated_user.id
    let organization_id_to_join = pending_invite.organization_id;
    let role_to_assign = pending_invite.role_to_assign;
    let invited_by_user_id = pending_invite.invited_by_user_id;

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            log::error!("Failed to begin transaction for accepting invitation: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error. Please try again.".to_string() });
        }
    };

    // 5. Check if user is already an active member.
    match crate::queries::organizations::find_membership(&mut tx, organization_id_to_join, user_id_to_add).await {
        Ok(Some(existing_membership)) => {
            if existing_membership.status == OrganizationMemberStatus::Active.to_string() {
                log::info!(
                    "User {} is already an active member of org {}. Invitation ID {} acceptance redundant.",
                    user_id_to_add, organization_id_to_join, pending_invite.id
                );
                if let Err(e) = delete_pending_invitation(&mut tx, pending_invite.id).await {
                    log::error!("Failed to delete pending invitation {} after finding user already active: {}", pending_invite.id, e);
                }
                if let Err(e) = tx.commit().await {
                     log::error!("Failed to commit transaction after finding user already active: {e}");
                     return HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error. Please try again.".to_string() });
                }
                return HttpResponse::Ok().json(existing_membership);
            }
        }
        Ok(None) => { /* User not found in organization_members, proceed to add. */ }
        Err(e) => {
            log::error!("DB error checking existing membership for user {user_id_to_add} in org {organization_id_to_join}: {e}");
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Error verifying existing membership details.".to_string() });
        }
    }

    // 6. Add user to organization_members table.
    let new_member_record = match add_member(
        &mut tx, 
        organization_id_to_join, 
        user_id_to_add, 
        &role_to_assign, 
        OrganizationMemberStatus::Active.to_string().as_str(), 
        invited_by_user_id
    ).await {
        Ok(record) => record,
        Err(e) => {
            log::error!(
                "Failed to add member (user {}, org {}) after accepting invite ID {}: {}",
                user_id_to_add, organization_id_to_join, pending_invite.id, e
            );
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to activate membership.".to_string() });
        }
    };

    // 7. Delete the pending invitation.
    if let Err(e) = delete_pending_invitation(&mut tx, pending_invite.id).await {
        log::error!(
            "Failed to delete pending invitation {} after adding user {} to organization {}: {}. Proceeding with commit as main operation was successful.", 
            pending_invite.id, user_id_to_add, organization_id_to_join, e
        );
    }

    // 8. Commit the transaction.
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction for accepting invitation (ID {}): {}", pending_invite.id, e);
        return HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to finalize invitation acceptance.".to_string() });
    }

    HttpResponse::Ok().json(new_member_record)
} 
