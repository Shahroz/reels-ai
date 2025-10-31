//! Sends a magic link email to password-based users.
//!
//! Generates an email with a clickable magic link for passwordless authentication.
//! The link expires in 15 minutes. If POSTMARK_SERVER_TOKEN is not set, logs
//! the email details and returns Ok without sending.

use postmark::Query;

/// Sends a magic link authentication email.
///
/// # Arguments
///
/// * `postmark_client` - Shared Postmark client instance (from app state)
/// * `email` - The recipient's email address
/// * `magic_link` - The complete magic link URL with token
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Logging
///
/// - INFO: Magic link sent successfully
/// - ERROR: Failed to send email (production issue)
#[tracing::instrument(skip(postmark_client, magic_link))]
pub async fn send_magic_link_email(
    postmark_client: &postmark::reqwest::PostmarkClient,
    email: &str,
    magic_link: &str,
) -> anyhow::Result<()> {

    let from_email = std::env::var("POSTMARK_FROM_EMAIL")
        .unwrap_or_else(|_| "noreply@bounti.com".to_string());

    let body_html = format!(
        r#"<div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
<p>Hello,</p>
<p>Click the link below to sign in to your Bounti account.</p>
<p style="text-align: center; margin: 30px 0;">
    <a href="{}" style="background-color: #4F46E5; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">Sign In to Bounti</a>
</p>
<p><strong>This link will expire in 15 minutes</strong> for security.</p>
<p>If the button doesn't work, you can also copy and paste this link into your browser:</p>
<p style="word-break: break-all; color: #6B7280; font-size: 14px;">{}</p>
<p style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #E5E7EB;">If you didn't request this login link, you can safely ignore this email.</p>
<p>Best regards,<br>The Bounti Team</p>
</div>"#,
        magic_link,
        magic_link
    );

    let body_text = format!(
        "Hello,\n\nClick the link below to sign in to your Bounti account:\n\n{}\n\nThis link will expire in 15 minutes for security.\n\nIf you didn't request this login link, you can safely ignore this email.\n\nBest regards,\nThe Bounti Team",
        magic_link
    );

    let req = postmark::api::email::SendEmailRequest::builder()
        .from(from_email)
        .to(email)
        .subject("Your Login Link for Bounti")
        .body(postmark::api::Body::html(body_html))
        .build();

    req.execute(postmark_client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send magic link email: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("Postmark API error: {:?}", e))?;

    log::info!("Magic link email sent successfully to: {}", email);
    std::result::Result::Ok(())
}

// Note: Integration tests should mock or stub the Postmark client.
// Unit tests are not meaningful for external email service calls.

