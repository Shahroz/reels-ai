// Handler for fetching the current authenticated user's details.
// GET /api/users/me

use actix_web::{get, web, HttpResponse, Responder};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use crate::auth::tokens::Claims;
use crate::db::users::{find_user_by_id, PublicUser};
use crate::routes::error_response::ErrorResponse;
use crate::routes::users::me_response::{MeResponse, OrganizationCreditInfo};
use utoipa;
use log;

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "Current user details with organization credits retrieved successfully", body = MeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = []) 
    )
)]
#[get("/me")]
pub async fn get_current_user_handler(
    claims: web::ReqData<Claims>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = claims.user_id;

    // Fetch user details
    let user = match find_user_by_id(pool.get_ref(), user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::warn!("User not found for ID: {user_id}");
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "User not found.".to_string(),
            });
        }
        Err(e) => {
            log::error!("Database error trying to fetch user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user information.".to_string(),
            });
        }
    };

    // Fetch organization memberships with credit information
    let org_credits = match crate::queries::organizations::get_user_organizations_with_credits::get_user_organizations_with_credits(
        pool.get_ref(),
        user_id,
    ).await {
        Ok(orgs) => orgs
            .into_iter()
            .map(|org| OrganizationCreditInfo {
                organization_id: org.organization_id,
                organization_name: org.organization_name,
                credits_remaining: org.credits_remaining.unwrap_or(BigDecimal::from(0)),
                user_role: org.user_role,
                is_personal: org.is_personal,
            })
            .collect(),
        Err(e) => {
            log::error!("Database error fetching organization credits for user {user_id}: {e}");
            // Don't fail the entire request if org credits fetch fails
            // Just return empty organizations array
            Vec::new()
        }
    };

    // Check if user has unlimited access grant
    let is_unlimited = match crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(
        pool.get_ref(),
        user_id,
    ).await {
        Ok(has_unlimited) => has_unlimited,
        Err(e) => {
            log::error!("Error checking unlimited access for user {user_id}: {e}");
            // Default to false on error - safer to show credits UI than hide it
            false
        }
    };

    let public_user: PublicUser = user.into();
    let response = MeResponse {
        user: public_user,
        organizations: org_credits,
        is_unlimited,
    };

    HttpResponse::Ok().json(response)
} 