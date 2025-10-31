// Handler for listing pending invitations for the authenticated user.
// GET /api/invitations/pending

use crate::auth::tokens::Claims;
use crate::db::pending_invitations::{find_pending_invitations_for_email, PendingInvitationResponse};
use crate::db::users::find_user_by_id;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;

use utoipa;

// No request body for this GET request

#[utoipa::path(
    get,
    path = "/api/invitations/pending",
    responses(
        (status = 200, description = "List of pending invitations.", body = Vec<PendingInvitationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
#[get("/pending")]
pub async fn list_pending_invitations_handler(
    pool: web::Data<PgPool>,
    claims: Claims,
) -> impl Responder {
    let user_id = claims.user_id;
    log::info!("Attempting to list pending invitations for user_id: {user_id}");

    let user_email = match find_user_by_id(pool.get_ref(), user_id).await {
        Ok(Some(user)) => user.email,
        Ok(None) => {
            log::warn!("User not found with ID: {user_id} while listing pending invitations.");
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Authenticated user not found.".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to fetch user details for user_id {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user details.".to_string(),
            });
        }
    };

    match find_pending_invitations_for_email(pool.get_ref(), &user_email).await {
        Ok(invitations) => {
            log::info!("Successfully retrieved {} pending invitations for email {}: {:?}", invitations.len(), user_email, invitations);
            HttpResponse::Ok().json(invitations)
        }
        Err(e) => {
            log::error!("Failed to list pending invitations for email {user_email}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve pending invitations.".to_string(),
            })
        }
    }
} 