//! Sends guidance email to OAuth users who request magic links.
//!
//! Informs users that their account uses Google OAuth and directs them to
//! use the "Sign in with Google" button. If POSTMARK_SERVER_TOKEN is not set,
//! logs the email details and returns Ok without sending.

use postmark::Query;

/// Sends an email guiding OAuth users to use Google sign-in.
///
/// # Arguments
///
/// * `postmark_client` - Shared Postmark client instance (from app state)
/// * `email` - The recipient's email address
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Logging
///
/// - INFO: OAuth guidance email sent successfully
/// - ERROR: Failed to send email (production issue)
#[tracing::instrument(skip(postmark_client))]
pub async fn send_oauth_user_guidance_email(
    postmark_client: &postmark::reqwest::PostmarkClient,
    email: &str,
) -> anyhow::Result<()> {

    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "https://app.bounti.com".to_string());

    let from_email = std::env::var("POSTMARK_FROM_EMAIL")
        .unwrap_or_else(|_| "noreply@bounti.com".to_string());

    let body_html = format!(
        r#"<div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
<p>Hello,</p>
<p>We received a request to sign in to your Bounti account.</p>
<p>Your account is linked to Google. Please use the "Sign in with Google" button on our sign-in page:</p>
<p style="text-align: center; margin: 30px 0;">
    <a href="{}/sign-in" style="background-color: #4F46E5; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">Go to Sign In Page</a>
</p>
<p>If the button doesn't work, you can copy and paste this link into your browser:</p>
<p style="word-break: break-all; color: #6B7280; font-size: 14px;">{}/sign-in</p>
<p style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #E5E7EB;">If you didn't request this, you can safely ignore this email.</p>
<p>Best regards,<br>The Bounti Team</p>
</div>"#,
        frontend_url,
        frontend_url
    );

    let body_text = format!(
        "Hello,\n\nWe received a request to sign in to your Bounti account.\n\nYour account is linked to Google. Please use the \"Sign in with Google\" button on our sign-in page:\n\n{}/sign-in\n\nIf you didn't request this, you can safely ignore this email.\n\nBest regards,\nThe Bounti Team",
        frontend_url
    );

    let req = postmark::api::email::SendEmailRequest::builder()
        .from(from_email)
        .to(email)
        .subject("Sign in to Bounti with Google")
        .body(postmark::api::Body::html(body_html))
        .build();

    req.execute(postmark_client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send OAuth guidance email: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("Postmark API error: {:?}", e))?;

    log::info!("OAuth guidance email sent successfully to: {}", email);
    std::result::Result::Ok(())
}

// Note: Integration tests should mock or stub the Postmark client.
// Unit tests are not meaningful for external email service calls.

