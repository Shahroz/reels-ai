//! Handles the HTTP DELETE request to revoke a specific API key.
//!
//! This endpoint revokes (deletes) an API key identified by its ID.
//! It verifies that the key belongs to the authenticated user before deletion.
//! Extracts user ID from claims and key ID from the path.
//! Adheres to coding standards: one function per file, fully qualified paths.
use tracing::instrument;

/// Deletes (revokes) a specific API key.
/// 
/// For admin users: Can delete any API key regardless of ownership.
/// For non-admin users: Can only delete their own API keys.
#[utoipa::path(
    delete,
    path = "/api/keys/{key_id}",
    tag = "API Keys",
    params(
        ("key_id" = String, Path, description = "ID of the API key to delete", format = "uuid")
    ),
    responses(
        (status = 204, description = "API Key deleted successfully"),
        (status = 401, description = "User not authenticated"),
        (status = 403, description = "Forbidden - Key does not belong to user or not found"),
        (status = 404, description = "API Key not found"), // Merged 403/404 logic
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("user_auth" = []) // Indicate JWT auth is required
    )
)]
#[actix_web::delete("/{key_id}")] // Fully qualified attribute
#[instrument(skip(pool, claims))]
pub async fn delete_key_handler(
    pool: actix_web::web::Data<sqlx::PgPool>, // Fully qualified paths
    key_id: actix_web::web::Path<uuid::Uuid>, // Fully qualified paths
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>, // Fully qualified paths
) -> impl actix_web::Responder {
    // Fully qualified trait
    let user_id = claims.user_id; // Get user_id from claims
    let is_admin = claims.is_admin; // Check if user is admin
    let key_id_val = key_id.into_inner();

    let result = if is_admin {
        // Admin users can delete any API key
        crate::db::api_keys::delete_any_api_key(&pool, key_id_val).await
    } else {
        // Non-admin users can only delete their own API keys
        crate::db::api_keys::delete_api_key(&pool, user_id, key_id_val).await
    };

    match result {
        // Fully qualified path
        Ok(true) => actix_web::HttpResponse::NoContent().finish(), // Fully qualified path
        Ok(false) => {
            actix_web::HttpResponse::Forbidden().json("API key not found or access denied")
        } // Fully qualified path
        Err(e) => {
            log::error!(
                // log macro
                "Failed to delete API key {key_id_val} for user {user_id}: {e}"
            );
            actix_web::HttpResponse::InternalServerError().json("Failed to delete API key")
            // Fully qualified path
        }
    }
}

#[cfg(test)] // Changed from FALSE
mod tests {
    // Basic compilation check and placeholder.

    #[test]
    fn handler_compiles() {
        let _ = super::delete_key_handler;
        std::assert!(true);
    }
}