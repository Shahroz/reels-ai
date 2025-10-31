//! Handles magic link request for OAuth users.
//!
//! OAuth users don't have passwords, so they receive a guidance email
//! directing them to use the Google sign-in flow instead of magic links.

/// Sends guidance email to OAuth user and logs the event.
///
/// # Arguments
///
/// * `postmark_client` - Shared Postmark client instance
/// * `user` - The OAuth user (has no password_hash)
///
/// # Returns
///
/// Returns Ok(()) - errors are logged but don't prevent success response
pub async fn handle_oauth_user_magic_link_request(
    postmark_client: &postmark::reqwest::PostmarkClient,
    user: &crate::db::users::User,
) {
    log::info!("Sending OAuth guidance email to user: {}", user.id);
    
    if let std::result::Result::Err(e) = 
        crate::services::email_service::send_oauth_user_guidance_email(postmark_client, &user.email).await 
    {
        log::error!("Failed to send OAuth guidance email to user {}: {}", user.id, e);
    } else {
        log::info!("OAuth guidance email sent successfully to user: {}", user.id);
    }
}

