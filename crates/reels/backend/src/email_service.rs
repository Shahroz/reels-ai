// backend/src/email_service.rs

use anyhow::{Context, Result};
use postmark::reqwest::PostmarkClient;
use postmark::Query; // Import the Query trait for execute()
use std::env;
use uuid::Uuid;
// Import the correct error types from the postmark crate
use postmark::QueryError; // QueryError is re-exported at the crate root
use postmark::reqwest::PostmarkClientError; // This path should be correct
use tracing::instrument;
use url::Url;

// Helper function to get Postmark client
// This function still returns a Result, failure indicates inability to build client (e.g. missing key)
fn get_postmark_client() -> Result<PostmarkClient> {
    let api_key =
        env::var("POSTMARK_SERVER_TOKEN").context("POSTMARK_SERVER_TOKEN environment variable not set")?;
    // Corrected: Use the builder pattern as shown in postmark-rs docs
    Ok(PostmarkClient::builder().server_token(api_key).build())
}

/// Sends a verification email to the user.
/// Assumes a Postmark template with alias "email-verification" exists.
#[instrument(skip(client, token))]
pub async fn send_verification_email(
    client: &PostmarkClient,
    _user_id: Uuid, // Prefixed unused variable
    email: &str,
    token: &str,
) -> Result<()> {
    let from_email = env::var("POSTMARK_FROM_EMAIL").unwrap_or_else(|_| "support@example.com".to_string());
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()); // Base URL for verification link
    let verification_link = format!("{frontend_url}/verify-email?token={token}");

    {
            let product_name = "Bounti";
            let subject = format!("[{product_name}] Verify Your Email Address");
            let html_body = format!(
                r#"<p>Hello,</p>
<p>Thanks for signing up for {}! Please click the link below to verify your email address.</p>
<p><a href="{}">Verify Email</a></p>
<p>If you did not sign up for an account, you can safely ignore this email.</p>
<p>Best regards,<br>The {} Team</p>"#,
                product_name,
                verification_link.clone(),
                product_name
            );

        // Build the request
        let req = postmark::api::email::SendEmailRequest::builder()
            .from(from_email.clone()) // Clone for logging if needed
            .to(email)
            .subject(subject)
            .body(postmark::api::Body::html(html_body))
            .build();

        // Execute the request
        req.execute(client)
            .await
            .context("Failed to send verification email via Postmark")?
            .error_for_status() // Check Postmark API response for errors
            .map_err(|e| anyhow::anyhow!("Postmark API returned an error: {:?}", e)) // Map error to anyhow::Error
            .context("Postmark API returned an error for verification email")?;

        log::info!("Verification email sent to {email}");
    }
    Ok(())
}

/// Sends a password reset email to the user.
/// Assumes a Postmark template with alias "password-reset" exists.
#[instrument(skip(client, token))]
pub async fn send_password_reset_email(
    client: &PostmarkClient,
    _user_id: Uuid, // Prefixed unused variable
    email: &str,
    token: &str,
) -> Result<()> {
    let from_email = env::var("POSTMARK_FROM_EMAIL").unwrap_or_else(|_| "support@example.com".to_string());
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    // Extract just the origin (protocol + domain + port) without any path
    // Password reset is at the root level, not under /real-estate or other paths
    let base_url = if let Ok(parsed) = Url::parse(&frontend_url) {
        format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or("localhost"))
            + &parsed.port().map(|p| format!(":{}", p)).unwrap_or_default()
    } else {
        frontend_url.trim_end_matches('/').to_string()
    };
    
    let reset_link = format!("{base_url}/reset-password?token={token}");

   {
           let product_name = "Bounti";
           let subject = format!("[{product_name}] Password Reset Request");
           let html_body = format!(
               r#"<div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
    <p>Hello,</p>
    <p>You recently requested to reset your password for your {} account. Click the button below to create a new password.</p>
    <p style="text-align: center; margin: 30px 0;">
        <a href="{}" style="background-color: #4F46E5; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">Reset Password</a>
    </p>
    <p><strong>This link will expire in 1 hour</strong> for security reasons.</p>
    <p>If the button doesn't work, you can also copy and paste this link into your browser:</p>
    <p style="word-break: break-all; color: #6B7280; font-size: 14px;">{}</p>
    <p style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #E5E7EB;">If you did not request a password reset, please ignore this email or contact support if you have concerns.</p>
    <p>Best regards,<br>The {} Team</p>
</div>"#,
               product_name,
               reset_link.clone(),
               reset_link.clone(),
               product_name
           );

       // Build the request
       let req = postmark::api::email::SendEmailRequest::builder()
            .from(from_email.clone()) // Clone for logging if needed
            .to(email)
            .subject(subject)
            .body(postmark::api::Body::html(html_body))
            .build();

        // Execute the request
        req.execute(client)
            .await
            .context("Failed to send password reset email via Postmark")?
            .error_for_status() // Check Postmark API response for errors
            .map_err(|e| {
                log::error!("Postmark API error: {e:?}");
                anyhow::anyhow!("Postmark API returned an error: {:?}", e)
            }) // Map error to anyhow::Error
            .context("Postmark API returned an error for password reset email")?;

        log::info!("Password reset email sent to {email}");
    }
    Ok(())
}

// TODO: Add other email sending functions (e.g., welcome email, notification emails)
// Ensure any new functions also handle the missing API key case if they rely on get_postmark_client().

// Basic error type for the email service
#[derive(Debug)]
pub enum EmailError {
    ConfigurationError(String),
    MessageBuildError(String),
    PostmarkError(QueryError<PostmarkClientError>),
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::ConfigurationError(s) => write!(f, "Email configuration error: {s}"),
            EmailError::MessageBuildError(s) => write!(f, "Email message build error: {s}"),
            EmailError::PostmarkError(e) => write!(f, "Postmark API error: {e}"),
        }
    }
}

// Correct From implementation for the new PostmarkError variant
impl From<QueryError<PostmarkClientError>> for EmailError {
    fn from(err: QueryError<PostmarkClientError>) -> Self {
        EmailError::PostmarkError(err)
    }
}

pub async fn send_invitation_email(
    client: &PostmarkClient,
    recipient_email: &str,
    recipient_name: Option<&str>,
    organization_name: &str,
    invitation_token: &str,
) -> Result<(), EmailError> {
    let from_email = env::var("POSTMARK_FROM_EMAIL").map_err(|_| {
        EmailError::ConfigurationError("POSTMARK_FROM_EMAIL environment variable not set".to_string())
    })?;

    let frontend_url_base =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let accept_link =
        format!("{frontend_url_base}/handle_invitation?token={invitation_token}");

    let subject = format!("You're invited to join {organization_name} on Bounti!");

    // Styled HTML email with Bounti brand colors
    // Colors from Bounti brand palette:
    // - Flame Orange: #EE5936 (primary CTA)
    // - Midnight: #1A1817 (text)
    // - Gray: #464343 (secondary text)
    // - Cadet Blue: #629FA7 (links)
    // - Off-White: #F9FAF5 (background)
    // - White: #FFFFFF (container)
    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #1A1817;
            margin: 0;
            padding: 0;
            background-color: #F9FAF5;
            width: 100%;
        }}
        .email-wrapper {{
            width: 100%;
            background-color: #F9FAF5;
            padding: 40px 20px;
        }}
        .container {{
            background-color: #FFFFFF;
            border-radius: 8px;
            padding: 60px 40px;
            margin: 0 auto;
            max-width: 600px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }}
        .logo {{
            text-align: center;
            margin-bottom: 40px;
        }}
        .content {{
            font-size: 18px;
            margin-bottom: 30px;
            text-align: center;
            color: #1A1817;
        }}
        .content p {{
            margin: 0 0 16px 0;
        }}
        .button-container {{
            text-align: center;
            margin: 40px 0;
        }}
        .button {{
            display: inline-block;
            background-color: #EE5936;
            color: #FFFFFF !important;
            text-decoration: none;
            padding: 16px 48px;
            border-radius: 8px;
            font-size: 18px;
            font-weight: 600;
            text-align: center;
        }}
        .fallback-link {{
            font-size: 14px;
            color: #464343;
            margin-top: 30px;
            text-align: center;
        }}
        .fallback-link p {{
            margin: 8px 0;
        }}
        .fallback-link a {{
            color: #629FA7;
            word-break: break-all;
        }}
        .signature {{
            margin-top: 50px;
            text-align: center;
            color: #1A1817;
        }}
        .signature p {{
            margin: 8px 0;
        }}
        .footer {{
            text-align: center;
            font-size: 12px;
            color: #464343;
            margin-top: 40px;
            padding: 30px 20px 0;
            max-width: 600px;
            margin-left: auto;
            margin-right: auto;
        }}
        .footer p {{
            margin: 8px 0;
        }}
        .footer a {{
            color: #629FA7;
            text-decoration: none;
        }}
    </style>
</head>
<body>
    <div class="email-wrapper">
        <div class="container">
            <div class="logo">
                <table border="0" cellpadding="0" cellspacing="0" style="margin: 0 auto;">
                    <tr>
                        <td style="padding: 0; text-align: center;">
                            <span style="font-size: 32px; color: #EE5936;">🏠</span>
                            <span style="font-family: Arial, sans-serif; font-size: 32px; font-weight: bold; color: #1A1817; margin-left: 8px;">bounti</span>
                        </td>
                    </tr>
                </table>
            </div>
            
            <div class="content">
                <p>You've been invited to your team {} organization. We're excited to have you.</p>
                <p>Click the button below to accept your invite and join your fellow agents.</p>
            </div>
            
            <div class="button-container">
                <a href="{}" class="button">Accept Invite</a>
            </div>
            
            <div class="fallback-link">
                <p>If that button doesn't work use this link:</p>
                <p><a href="{}">{}</a></p>
            </div>
            
            <div class="signature">
                <p>Thanks,</p>
                <p><strong>The Bounti team</strong></p>
            </div>
        </div>
        
        <div class="footer">
            <p>Bounti Labs, 2099 Gateway Place, Suite 560, San Jose, CA 95110, United States</p>
            <p><a href="https://bounti.ai/unsubscribe">Unsubscribe</a> | <a href="https://bounti.ai/preferences">Manage preferences</a></p>
        </div>
    </div>
</body>
</html>"#,
        organization_name,
        accept_link,
        accept_link,
        accept_link
    );

    let email_request = postmark::api::email::SendEmailRequest::builder()
        .from(from_email.clone()) // Clone from_email for logging
        .to(recipient_email)
        .subject(subject)
        .body(postmark::api::Body::html(html_body))
        .build();

    match email_request.execute(client).await {
        Ok(response) => {
            // Postmark API errors are often indicated by ErrorCode in the response body
            // A 0 ErrorCode typically means success.
            if response.error_code == 0 {
                log::info!("Invitation email successfully sent to {} via Postmark. MessageID: {:?}", recipient_email, response.message_id);
                Ok(())
            } else {
                log::error!(
                    "Postmark reported an error for {}: ErrorCode: {}, Message: {}. Full Response: {:?}",
                    recipient_email,
                    response.error_code,
                    response.message,
                    response
                );
                Err(EmailError::MessageBuildError(format!(
                    "Postmark error: ErrorCode: {}, Message: {}",
                    response.error_code,
                    response.message
                )))
            }
        }
        Err(e) => {
            log::error!("Failed to send invitation email to {recipient_email} via Postmark: {e:?}");
            Err(EmailError::from(e))
        }
    }
}
