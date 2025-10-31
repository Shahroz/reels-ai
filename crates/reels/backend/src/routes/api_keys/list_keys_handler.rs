//! Handles the HTTP GET request to list API keys for the authenticated user.
//!
//! This endpoint retrieves metadata for all active API keys associated with the user.
//! It extracts the user ID from JWT claims and queries the database.
//! Returns a list of API key metadata objects with user details.
//! Supports optional search filtering by user email (case insensitive).
//! Follows coding standards: one function per file, fully qualified paths.

use crate::routes::api_keys::list_keys_params::ListApiKeysParams;
use crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails;
use tracing::instrument;

/// Lists API keys for the authenticated user.
/// 
/// For admin users: Returns all API keys in the system.
/// For non-admin users: Returns only their own API keys.
/// Supports optional search filtering by user email (case insensitive).
#[utoipa::path(
    get,
    path = "/api/keys",
    tag = "API Keys",
    params(
        ListApiKeysParams
    ),
    responses(
        (status = 200, description = "List of API key metadata with user details", body = [ApiKeyWithUserDetails]),
        (status = 401, description = "User not authenticated"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("user_auth" = [])
    )
)]
#[actix_web::get("")] // Fully qualified attribute
#[instrument(skip(pool, claims))]
pub async fn list_keys_handler(
    pool: actix_web::web::Data<sqlx::PgPool>, // Fully qualified paths
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>, // Fully qualified paths
    query: actix_web::web::Query<ListApiKeysParams>, // Use the renamed params struct
) -> impl actix_web::Responder {
    // Fully qualified trait
    let user_id = claims.user_id; // Get user_id from claims
    let is_admin = claims.is_admin; // Check if user is admin
    let search_term = query.search.as_deref(); // Extract search term

    let result = if is_admin {
        // Admin users can see all API keys with user details
        crate::db::api_keys::list_all_api_keys_with_user_details_search(&pool, search_term).await
    } else {
        // Non-admin users can only see their own API keys with user details
        crate::db::api_keys::list_api_keys_with_user_details_for_user_search(&pool, user_id, search_term).await
    };

    match result {
        // Fully qualified path
        Ok(keys) => actix_web::HttpResponse::Ok().json(keys), // Fully qualified path
        Err(e) => {
            log::error!("Failed to list API keys for user {user_id}: {e}"); // log macro
            actix_web::HttpResponse::InternalServerError().json("Failed to list API keys")
            // Fully qualified path
        }
    }
}

#[cfg(test)] // Changed from FALSE
mod tests {
    // Basic compilation check and placeholder.

    #[test]
    fn handler_compiles() {
        let _ = super::list_keys_handler;
        std::assert!(true);
    }
}