//! Creates session JWT token for authenticated user.
//!
//! Generates a long-lived session token after successful magic link verification.

/// Creates a session JWT token for the user.
///
/// # Arguments
///
/// * `user` - The authenticated user
///
/// # Returns
///
/// `Ok(token)` containing the session JWT string
/// `Err(response)` if token generation fails
pub fn create_session_token(
    user: &crate::db::users::User,
) -> std::result::Result<std::string::String, actix_web::HttpResponse> {
    log::debug!("Generating session JWT for user: {}", user.id);

    let session_token =
        match crate::routes::auth::generate_session_token_for_user::generate_session_token_for_user(
            user,
        ) {
            std::result::Result::Ok(t) => t,
            std::result::Result::Err(e) => {
                log::error!("Failed to create session JWT for user {}: {}", user.id, e);
                return std::result::Result::Err(
                    actix_web::HttpResponse::InternalServerError().body("Authentication failed"),
                );
            }
        };

    log::debug!("Session JWT created for user: {}", user.id);

    std::result::Result::Ok(session_token)
}

