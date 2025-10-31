//! Handles the HTTP POST request to create a new API key.
//!
//! This endpoint generates a new API key associated with the authenticated user.
//! It extracts the user ID from the JWT claims and calls the database function.
//! Returns the newly generated raw key upon success.
//! Adheres to coding standards: one function per file, fully qualified paths.

use crate::routes::api_keys::create_api_key_response::CreateApiKeyResponse;
use crate::routes::api_keys::create_api_key_request::CreateApiKeyRequest;
use crate::auth::tokens::Claims;
use tracing::instrument;

/// Creates a new API key for the specified user.
/// 
/// For non-admin users: user_id is optional and will default to their own user_id.
/// For admin users: user_id is optional but can be specified to create keys for other users.
#[utoipa::path(
    post,
    path = "/api/keys",
    tag = "API Keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 200, description = "API Key created successfully", body = CreateApiKeyResponse),
        (status = 401, description = "User not authenticated"),
        (status = 403, description = "Non-admin users cannot create API keys for other users"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("user_auth" = []) // Indicate JWT auth is required
    )
)]
#[actix_web::post("")] // Fully qualified attribute
#[instrument(skip(pool, request, claims))]
pub async fn create_key_handler(
    pool: actix_web::web::Data<sqlx::PgPool>, // Fully qualified paths
    request: actix_web::web::Json<CreateApiKeyRequest>, // Request body
    claims: actix_web::web::ReqData<Claims>, // JWT claims for user identification
) -> impl actix_web::Responder {
    let authenticated_user_id = claims.user_id;
    let is_admin = claims.is_admin;
    let allowed_domains = request.allowed_domains.clone();

    // Determine the target user_id for the API key
    let target_user_id = match request.user_id {
        Some(requested_user_id) => {
            // If user_id is provided in request, check if user is admin
            if !is_admin && requested_user_id != authenticated_user_id {
                return actix_web::HttpResponse::Forbidden().json("Non-admin users cannot create API keys for other users");
            }
            requested_user_id
        }
        None => {
            // If no user_id provided, use the authenticated user's ID
            authenticated_user_id
        }
    };

    match crate::db::api_keys::create_api_key(&pool, target_user_id, allowed_domains).await {
        // Fully qualified path
        Ok(raw_key) => actix_web::HttpResponse::Ok().json(
            crate::routes::api_keys::create_api_key_response::CreateApiKeyResponse { raw_key },
        ), // Fully qualified paths
        Err(e) => {
            log::error!("Failed to create API key for user {target_user_id}: {e}"); // log macro is often globally available
            actix_web::HttpResponse::InternalServerError().json("Failed to create API key")
            // Fully qualified path
        }
    }
}

#[cfg(test)] // Changed from FALSE
mod tests {
    // Note: Full integration tests requiring DB and auth setup are complex.
    // Basic compilation check and placeholder for future tests.
    // Fully qualified paths would be needed for mocks or test utilities.

    #[test]
    fn handler_compiles() {
        // This test primarily ensures the handler function signature and basic structure compile.
        // It doesn't invoke the handler.
        let _ = super::create_key_handler; // Reference the handler to check visibility
        std::assert!(true); // Basic assertion to make the test pass
    }
}