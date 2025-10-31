//! Handles magic link request for password-based users.
//!
//! Generates a magic link JWT token and sends it to the user's email.
//! This is the core magic link authentication flow.

/// Generates and sends magic link to password user.
///
/// # Arguments
///
/// * `postmark_client` - Shared Postmark client instance
/// * `user` - The password user (has password_hash)
/// * `return_url` - Optional URL to redirect to after successful authentication
///
/// # Returns
///
/// Returns Ok(()) - errors are logged but don't prevent success response
pub async fn handle_password_user_magic_link_request(
    postmark_client: &postmark::reqwest::PostmarkClient,
    user: &crate::db::users::User,
    return_url: std::option::Option<&str>,
) {
    log::debug!("Generating magic link for user: {}", user.id);
    
    match crate::auth::tokens::generate_magic_link_jwt(user) {
        std::result::Result::Ok(token) => {
            // Get FRONTEND_URL
            let frontend_url = std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string());
            
            // Build magic link using extracted pure function (testable!)
            let magic_link = crate::routes::auth::build_magic_link_url::build_magic_link_url(
                &frontend_url,
                &token,
                return_url,
            );
            
            log::debug!("Sending magic link email to user: {}", user.id);
            
            if let std::result::Result::Err(e) = 
                crate::services::email_service::send_magic_link_email(postmark_client, &user.email, &magic_link).await 
            {
                log::error!("Failed to send magic link email to user {}: {}", user.id, e);
            } else {
                log::info!("Magic link sent successfully to user: {}", user.id);
            }
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to generate magic link JWT for user {}: {}", user.id, e);
        }
    }
}

