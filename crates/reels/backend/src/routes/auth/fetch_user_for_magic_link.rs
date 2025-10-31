//! Fetches user for magic link authentication.
//!
//! Retrieves user from database and validates their status for login.
//! Returns user on success or appropriate error response.

/// Fetches and validates user for magic link authentication.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `user_id` - UUID of the user from JWT claims
///
/// # Returns
///
/// `Ok(user)` if user exists and can authenticate
/// `Err(response)` with appropriate HTTP error response
///
/// # Security
///
/// - Rejects deactivated users
/// - Returns generic errors to prevent user enumeration
pub async fn fetch_user_for_magic_link(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> std::result::Result<crate::db::users::User, actix_web::HttpResponse> {
    // Look up user
    let user = match crate::user_management::find_user_by_id(pool, user_id).await {
        std::result::Result::Ok(std::option::Option::Some(u)) => u,
        std::result::Result::Ok(std::option::Option::None) => {
            log::warn!("User not found for magic link: {}", user_id);
            return std::result::Result::Err(
                actix_web::HttpResponse::Unauthorized().body("User not found"),
            );
        }
        std::result::Result::Err(e) => {
            log::error!(
                "Database error during magic link verification for user {}: {}",
                user_id,
                e
            );
            return std::result::Result::Err(
                actix_web::HttpResponse::InternalServerError().body("Authentication failed"),
            );
        }
    };

    log::debug!(
        "User found: {} (token_version: {})",
        user.id,
        user.token_version
    );

    // Check user status
    if user.status == "deactivated" {
        log::warn!(
            "Deactivated user {} attempted login via magic link",
            user.id
        );
        return std::result::Result::Err(
            actix_web::HttpResponse::Unauthorized().body("Account is deactivated"),
        );
    }

    std::result::Result::Ok(user)
}

