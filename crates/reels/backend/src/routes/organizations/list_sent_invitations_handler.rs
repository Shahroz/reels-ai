use chrono::{DateTime, Utc};

use actix_web::{web, HttpResponse, Responder, get};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use crate::auth::permissions::check_active_membership;
use crate::queries::pending_invitations;

#[derive(Serialize, ToSchema, Debug)]
pub struct SentInvitationDetails {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid")]
    pub id: Uuid,
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1")]
    pub organization_id: Uuid,
    #[schema(example = "invitee@example.com")]
    pub invited_email: String,
    #[schema(example = "member")]
    pub role_to_assign: String,
    #[schema(example = "invited")]
    pub status: String,
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = Option<String>)]
    pub invited_by_user_id: Option<Uuid>,
    #[schema(value_type = String, format = "date-time")]
    pub invited_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ListSentInvitationsApiResponse {
    invitations: Vec<SentInvitationDetails>,
}

/// Lists pending invitations sent out by a specific organization.
///
/// This endpoint retrieves all membership records for the given organization
/// that have a status of "invited".
///
/// Requires authentication and membership (e.g. owner or admin role) in the organization.
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/sent-invitations",
    tag = "Organizations",
    params(
        ("org_id" = Uuid, Path, description = "The ID of the organization whose sent invitations are to be listed.")
    ),
    responses(
        (status = 200, description = "Successfully retrieved list of sent invitations.", body = ListSentInvitationsApiResponse),
        (status = 401, description = "Unauthorized.", body = ErrorResponse),
        (status = 403, description = "Forbidden. User may not have permissions or is not part of the organization.", body = ErrorResponse),
        (status = 404, description = "Organization not found.", body = ErrorResponse),
        (status = 500, description = "Internal Server Error.", body = ErrorResponse)
    ),
    security(
        ("token" = [])
    )
)]
#[get("/{org_id}/sent-invitations")]
pub async fn list_sent_invitations_handler(
    path: web::Path<Uuid>,
    claims: Claims,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let organization_id = path.into_inner();
    let current_user_id = claims.user_id;
    log::info!(
        "[LIST_SENT_INVITES] Handler entered for org_id: {organization_id}, user_id: {current_user_id}"
    );

    // 1. Check for active membership of the requesting user in the organization
    if let Err(response) =
        check_active_membership(pool.get_ref(), organization_id, current_user_id).await
    {
        log::warn!(
            "[LIST_SENT_INVITES] Membership check failed for user_id: {} in org_id: {}. Response: {:?}",
            current_user_id, organization_id, response.status()
        );
        return response;
    }
    // If we reach here, the user is an active member.
    log::info!(
        "[LIST_SENT_INVITES] User {current_user_id} is an active member of org {organization_id}. Proceeding to fetch invitations."
    );

    log::info!(
       "[LIST_SENT_INVITES] Attempting to find invitations for organization_id: {organization_id} from pending_invitations table."
   );
   match pending_invitations::find_pending_invitations_for_organization::find_pending_invitations_for_organization(
       pool.get_ref(), 
       organization_id,
   )
    .await
    {
        Ok(db_invitations) => {
            log::info!(
                "[LIST_SENT_INVITES] Successfully fetched {} DB rows from pending_invitations for org_id: {}.",
                db_invitations.len(), organization_id
            );

            let api_invitations: Vec<SentInvitationDetails> = db_invitations
                .into_iter()
                .map(|db_row| SentInvitationDetails {
                    id: db_row.id,
                    organization_id: db_row.organization_id,
                    invited_email: db_row.invited_email,
                    role_to_assign: db_row.role_to_assign,
                    status: "invited".to_string(),
                    invited_by_user_id: db_row.invited_by_user_id,
                    invited_at: db_row.created_at,
                    expires_at: db_row.token_expires_at,
                })
                .collect();

            HttpResponse::Ok().json(ListSentInvitationsApiResponse { invitations: api_invitations })
        }
        Err(e) => {
            log::error!(
                "[LIST_SENT_INVITES] Error fetching sent invitations from pending_invitations for org_id: {organization_id}: {e}"
            );
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to fetch sent invitations: {e}"),
            })
        }
    }
} 
