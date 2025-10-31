// Handler for inviting a user to an organization.

//POST /api/organizations/{org_id}/members


use crate::auth::tokens::Claims;
use crate::db::users::find_user_by_email;
use crate::db::pending_invitations;
use crate::queries::organizations::{find_membership, find_organization_by_id};
use crate::auth::invitation_tokens::generate_invitation_token;
use crate::email_service::send_invitation_email;
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use utoipa::ToSchema;
use crate::auth::tokens::get_jwt_secret;
use log;
use validator::Validate;
use chrono::{Utc, Duration};

#[derive(Debug, Deserialize, ToSchema, Validate, serde::Serialize)]
pub struct InviteMemberRequest {
    #[schema(example = "user_to_invite@example.com")]
    #[validate(email)]
    pub email: String,
    // Role is implicitly "member" for invitations in this handler for now.
    // pub role: Option<String>, // Future: Allow inviting with specific roles
}

#[utoipa::path(
    post,
    path = "/api/organizations/{organization_id}/members",
    tag = "Organizations",
    request_body = InviteMemberRequest,
    params(
        ("organization_id" = Uuid, Path, description = "The ID of the organization to invite a member to")
    ),
    responses(
        (status = 201, description = "Invitation sent successfully", body = pending_invitations::PendingInvitation),
        (status = 400, description = "Invalid request (e.g., bad email, inviting self)", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (e.g., not an owner, organization not found, or trying to invite owner)"),
        (status = 404, description = "Organization not found", body = ErrorResponse),
        (status = 409, description = "User already a member or an invitation has already been sent", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn invite_member_handler(
    claims: Claims,
    organization_id_path: web::Path<Uuid>,
    payload: web::Json<InviteMemberRequest>,
    pool: web::Data<PgPool>,
    postmark_client: web::Data<std::sync::Arc<postmark::reqwest::PostmarkClient>>,
) -> Result<HttpResponse, ErrorResponse> {
    let inviter_user_id = claims.user_id;
    let organization_id = organization_id_path.into_inner();

    if let Err(validation_errors) = payload.validate() {
        log::warn!("InviteMemberRequest validation failed: {validation_errors:?}");
        let error_message = validation_errors.to_string();
        return Ok(HttpResponse::BadRequest().json(ErrorResponse { error: error_message }));
    }

    let recipient_email = payload.0.email.trim().to_lowercase();

    // 1. Fetch organization details
    let organization = match find_organization_by_id(&pool, organization_id).await {
        Ok(Some(org)) => {
            if org.owner_user_id != inviter_user_id {
                return Ok(HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Only the organization owner can invite new members.".to_string(),
                }));
            }
            // Check if organization is a personal organization
            if org.is_personal {
                return Ok(HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Members cannot be added to personal organizations. Personal organizations are for individual use only.".to_string(),
                }));
            }
            org
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ErrorResponse {
                error: "Organization not found.".to_string(),
            }));
        }
        Err(e) => {
            log::error!("DB error fetching organization {organization_id}: {e}");
            return Err(ErrorResponse { error: format!("DB error fetching organization: {e}") });
        }
    };

    // 2. Check if an invitation already exists in pending_invitations
    match pending_invitations::find_pending_invitation_by_org_and_email(&pool, organization_id, &recipient_email).await {
        Ok(Some(_existing_pending_invitation)) => {
            log::warn!("Attempt to invite email {recipient_email} to org_id {organization_id} which already has a pending invitation.");
            return Ok(HttpResponse::Conflict().json(ErrorResponse {
                error: "An invitation has already been sent to this email address for this organization and is pending.".to_string(),
            }));
        }
        Ok(None) => { /* No pending invitation for this email, proceed */ }
        Err(e) => {
            log::error!("DB error checking for existing pending invitation for email {recipient_email}: {e}");
            return Err(ErrorResponse { error: format!("DB error checking existing invites: {e}") });
        }
    }

    // 3. Check details if the recipient_email belongs to an existing user
    let maybe_target_user = match find_user_by_email(&pool, &recipient_email).await {
        Ok(Some(user)) => Some(user),
        Ok(None) => None, // User does not exist, this is fine for sending an invitation
        Err(e) => {
            log::error!("DB error finding user by email {recipient_email}: {e}");
            // Not returning error here, as we can still send an invite if user lookup fails for some reason other than not found
            None
        }
    };

    if let Some(target_user) = &maybe_target_user {
        // Inviting self check
        if target_user.id == inviter_user_id {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: "You cannot invite yourself to the organization.".to_string(),
            }));
        }
        // Inviting owner check
        if target_user.id == organization.owner_user_id {
            return Ok(HttpResponse::Forbidden().json(ErrorResponse {
                error: "The organization owner is already part of the organization and cannot be invited again.".to_string(),
            }));
        }

        // Check if this existing user is already an active member
        // Note: This requires a transaction if we were to combine with other DB ops, but sticking to &pool for now.
        // A separate transaction for find_membership or handle potential race conditions if not using one.
        // For simplicity now, using pool directly.
        let mut tx_for_find_membership = pool.begin().await.map_err(|e| {
            log::error!("Failed to begin transaction for find_membership: {e}");
            ErrorResponse { error: format!("Database transaction error: {e}") }
        })?;
        match find_membership(&mut tx_for_find_membership, organization_id, target_user.id).await {
            Ok(Some(existing_membership)) => {
                // If user is already 'active' or even 'invited' (though pending_invitations check should catch new invites to same email)
                // For 'active' members, this is a clear conflict.
                // For 'invited' status in organization_members, it implies an older flow or an accepted invite somehow still marked.
                // The uq_pending_invitation on (organization_id, invited_email) should be the primary guard against duplicate *new* invites.
                log::warn!(
                    "Attempt to invite user_id {} (email {}) to org_id {} who already has membership status: {:?}. This should ideally be caught by pending_invitations check if status is 'invited'.",
                    target_user.id, recipient_email, organization_id, existing_membership.status
                );
                tx_for_find_membership.rollback().await.map_err(|e| {
                     log::error!("Failed to rollback transaction for find_membership: {e}");
                     ErrorResponse { error: format!("Database transaction error: {e}") }
                })?;
                return Ok(HttpResponse::Conflict().json(ErrorResponse {
                    error: format!(
                        "This user (email: {}) is already associated with the organization with status: {}.",
                        recipient_email, existing_membership.status
                    ),
                }));
            }
            Ok(None) => { /* User is not an active member, proceed with invitation */ }
            Err(e) => {
                log::error!("Failed to check existing membership for user {}: {}", target_user.id, e);
                tx_for_find_membership.rollback().await.map_err(|e| {
                     log::error!("Failed to rollback transaction for find_membership: {e}");
                     ErrorResponse { error: format!("Database transaction error: {e}") }
                })?;
                return Err(ErrorResponse { error: format!("DB error checking existing membership: {e}") });
            }
        }
        tx_for_find_membership.commit().await.map_err(|e| {
            log::error!("Failed to commit transaction for find_membership: {e}");
            ErrorResponse { error: format!("Database transaction error: {e}") }
        })?;
    }

    // 4. Generate an invitation token (JWT).
    let jwt_secret = match get_jwt_secret() {
        Ok(secret) => secret,
        Err(e) => {
            log::error!("JWT_SECRET not configured: {e}. Cannot generate invitation token.");
            // No transaction to rollback here as we are not in one globally for this handler yet.
            return Err(ErrorResponse {
                error: "Server configuration error preventing invitation generation.".to_string(),
            });
        }
    };

    let issuer = "narrativ.com"; // Placeholder - should be from config
    let audience = "narrativ_invitation"; // Placeholder - should be from config
    let token_duration_hours = 24 * 7; // Token valid for 7 days

    let invitation_token_str = match generate_invitation_token(
        organization_id, // This will be part of the claims
        &recipient_email, // This will be part of the claims
        "member", // role_to_assign, part of claims
        issuer,
        audience,
        &jwt_secret,
        token_duration_hours, // Pass duration in hours
    ) {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to generate invitation token: {e}");
            return Err(ErrorResponse {
                error: "Failed to prepare invitation. Please try again or contact support if the issue persists.".to_string(),
            });
        }
    };
    log::info!("Generated invitation token string for email {recipient_email}");

    // 5. Calculate token expiry for storing in DB
    let token_expires_at = Utc::now() + Duration::hours(token_duration_hours);

    // 6. Create a pending_invitation record with the token.
    log::info!(
        "Calling create_pending_invitation with: org_id={organization_id}, email={recipient_email}, role=\"member\", invited_by_user_id={inviter_user_id}, token_expires_at={token_expires_at}"
    );
    let pending_invitation_record = match pending_invitations::create_pending_invitation(
        &pool,
        organization_id,
        &recipient_email,
        "member", // role_to_assign
        &invitation_token_str,
        token_expires_at,
        Some(inviter_user_id),
    ).await {
        Ok(record) => record,
        Err(e) => {
            log::error!("Failed to create pending invitation for email {recipient_email}: {e}");
            // Check for unique constraint violation (organization_id, invited_email)
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    log::warn!("Unique constraint violation while creating pending invitation for email {recipient_email}: {db_err}");
                    return Ok(HttpResponse::Conflict().json(ErrorResponse {
                        error: "An invitation has already been sent to this email address for this organization.".to_string(),
                    }));
                }
            }
            return Err(ErrorResponse { error: format!("Failed to save invitation: {e}") });
        }
    };
    log::info!("create_pending_invitation returned record: {pending_invitation_record:?}");

    // 7. Send an invitation email.
    match send_invitation_email(
        &postmark_client,
        &recipient_email,
        None, // User's name is not available if they are not registered yet
        &organization.name,
        &invitation_token_str,
    )
    .await
    {
        Ok(_) => {
            log::info!(
                "Invitation email successfully sent to {recipient_email} for organization_id {organization_id}."
            );
        }
        Err(email_err) => {
            // Log the error but still consider the invitation created successfully in the DB.
            // The user can potentially retrieve the invite from a "pending invites" section in the UI later.
            log::error!(
                "Failed to send invitation email to {recipient_email} for org_id {organization_id}: {email_err:?}. Invitation record created successfully."
            );
        }
    }

    // 8. Return the created `pending_invitation` record.
    log::info!(
        "Invitation sent successfully to {recipient_email} for organization {organization_id} by user {inviter_user_id}. Pending invitation record: {pending_invitation_record:?}"
    );

    Ok(HttpResponse::Created().json(pending_invitation_record))
}