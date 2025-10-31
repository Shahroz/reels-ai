//! Handles listing organizations for the authenticated user.
//!
//! This endpoint retrieves and returns a list of all organizations
//! where the currently authenticated user is an active member.
//! It adheres to standard API practices for listing resources.

use crate::auth::tokens::Claims;
use crate::queries::organizations::list_organizations_for_user;
use crate::db::organizations::Organization;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize}; // Added for ListOrganizationsResponse
use utoipa::ToSchema; // Added for ListOrganizationsResponse
use sqlx::PgPool;
use utoipa; // Added for macro use

/// Response struct for listing organizations.
/// Wraps a vector of Organization structs, serializes transparently as an array.
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct ListOrganizationsResponse(pub Vec<Organization>);

/// Lists organizations the user is a member of.
///
/// Retrieves all organizations where the authenticated user has active membership.
#[utoipa::path(
    get,
    path = "/api/organizations",
    responses(
        (status = 200, description = "List of organizations user is a member of", body = ListOrganizationsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Organizations"
)]
#[get("")]
pub async fn list_organizations_handler(
    pool: web::Data<PgPool>,
    claims: Claims, // Changed from web::ReqData<Claims>
) -> impl Responder {
    let user_id = claims.user_id; // Get user ID from JWT claims

    match list_organizations_for_user(pool.get_ref(), user_id).await {
        Ok(organizations) => HttpResponse::Ok().json(ListOrganizationsResponse(organizations)),
        Err(e) => {
            log::error!("Failed to list organizations for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve organizations".to_string(),
            })
        }
    }
}