//! Handles the HTTP GET request for listing a user's bundles.
//!
//! This module defines the Actix-web handler responsible for processing
//! requests to retrieve all `ExpandedBundle` items associated with the authenticated user.
//! It extracts user information from authentication middleware and fetches
//! data from the database. Adheres to project coding standards, using fully
//! qualified paths for internal crate items and common `use` statements for
//! external crates like Actix-web and SQLx.

// Revision History (New File)
// - 2025-05-29T16:06:28Z @AI: Initial implementation of list_bundles handler.

use actix_web::{web, HttpResponse, Responder}; // Common Actix-web imports
use sqlx::PgPool; // For database pool type
use crate::routes::bundles::ListExpandedBundlesResponse;

// Other types are referred to by their fully qualified paths, e.g.,
// crate::middleware::auth::AuthenticatedUser, crate::db::bundles::Bundle, etc.

#[utoipa::path(
    get,
    path = "/api/bundles",
    responses(
        (status = 200, description = "List of expanded bundles retrieved successfully", body = ListExpandedBundlesResponse),
        (status = 401, description = "Unauthorized"), // Typically handled by middleware
        (status = 500, description = "Internal Server Error", body = crate::routes::error_response::ErrorResponse)
    ),
    tag = "Bundles",
    security(
        ("bearer_auth" = []) // Assumes "bearer_auth" is defined in ApiDoc's security_schemes
    )
)]
#[actix_web::get("")] // Registers this handler for GET requests at the root of its scope.
pub async fn list_bundles(
    pool: web::Data<PgPool>,
    user: web::ReqData<crate::middleware::auth::AuthenticatedUser>,
) -> impl Responder {
    let user_id = match user.into_inner() {
        crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
       crate::middleware::auth::AuthenticatedUser::ApiKey(id) => id,
   };

   // Default parameters for listing bundles
   let search_pattern = "%"; // match all names
   let sort_by = "created_at";
   let sort_order = "desc";
   let limit = 20;
   let offset = 0;
   match crate::queries::bundles::list_expanded_bundles_for_user::list_expanded_bundles_for_user(
       pool.get_ref(), user_id, search_pattern, sort_by, sort_order, limit, offset
   ).await {
       Ok(expanded_bundles) => {
            let response = crate::routes::bundles::list_bundles_response::ListExpandedBundlesResponse {
                items: expanded_bundles.clone(),
                total_count: expanded_bundles.len() as i64,
            };
            HttpResponse::Ok().json(response)
        }
       Err(sqlx_error) => {
           // Log the error for server-side observability.
            log::error!(
                "Database error while listing bundles for user_id {user_id}: {sqlx_error:?}"
            );
            // Return a generic error response to the client.
            HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Failed to retrieve bundles due to an internal server error."),
            })
        }
    }
}
