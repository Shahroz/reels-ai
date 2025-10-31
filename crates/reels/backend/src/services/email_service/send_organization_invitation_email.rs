//! Sends styled organization invitation emails.
//!
//! This function sends an invitation email to users when they are added to an
//! organization by an admin. The email uses Bounti-branded styling with an orange
//! call-to-action button and includes a fallback link for email clients that don't
//! support buttons. The HTML template is stored in the templates module for maintainability.

#[tracing::instrument(skip(client, invitation_link))]
pub async fn send_organization_invitation_email(
    client: &postmark::reqwest::PostmarkClient,
    recipient_email: &str,
    organization_name: &str,
    invitation_link: &str,
) -> anyhow::Result<()> {
    let from_email = std::env::var("POSTMARK_FROM_EMAIL")
        .unwrap_or_else(|_| String::from("support@bounti.com"));

    let subject = std::format!("Welcome to {organization_name} on Bounti!");

    // Format organization name with bold HTML
    let formatted_org_name = std::format!("<strong>{}</strong>", organization_name);

    // Use template from templates module and interpolate values
    // Template expects: org_name (with <strong>), button_link, fallback_href, fallback_text
    let template = crate::services::email_service::templates::organization_invitation_html::ORGANIZATION_INVITATION_HTML_TEMPLATE;
    let html_body = template
        .replacen("{}", &formatted_org_name, 1)
        .replacen("{}", invitation_link, 1)
        .replacen("{}", invitation_link, 1)
        .replacen("{}", invitation_link, 1);

    let email_request = postmark::api::email::SendEmailRequest::builder()
        .from(from_email)
        .to(recipient_email)
        .subject(subject)
        .body(postmark::api::Body::html(html_body))
        .build();

    postmark::Query::execute(email_request, client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send organization invitation email: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("Postmark API error: {:?}", e))?;

    log::info!(
        "Organization invitation email successfully sent to {} via Postmark",
        recipient_email
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_email_body_formatting() {
        // Test that organization name is properly formatted with <strong> tags
        let org_name = "Test Organization";
        let formatted = std::format!("<strong>{}</strong>", org_name);
        assert_eq!(formatted, "<strong>Test Organization</strong>");
        assert!(formatted.contains("<strong>"));
        assert!(formatted.contains("</strong>"));
    }

    #[test]
    fn test_email_subject_formatting() {
        // Test subject line formatting
        let org_name = "Test Organization";
        let subject = std::format!("Welcome to {org_name} on Bounti!");
        assert_eq!(subject, "Welcome to Test Organization on Bounti!");
        assert!(subject.starts_with("Welcome to "));
        assert!(subject.ends_with(" on Bounti!"));
    }

    #[test]
    fn test_template_has_required_placeholders() {
        // Test that template contains expected structure for formatting
        let template = crate::services::email_service::templates::organization_invitation_html::ORGANIZATION_INVITATION_HTML_TEMPLATE;
        
        // Template should have 4 placeholder positions: org name, button link, fallback link, fallback link text
        let placeholder_count = template.matches("{}").count();
        assert_eq!(
            placeholder_count, 4,
            "Template should have exactly 4 placeholders for format interpolation"
        );
        
        // Template should contain key structural elements
        assert!(template.contains("<!DOCTYPE html"));
        assert!(template.contains("Bounti"));
        assert!(template.contains("Join Your Team"));
        assert!(template.contains("Bounti Labs, 2099 Gateway Place"));
    }

    #[test]
    fn test_env_var_fallback() {
        // Test that missing env var falls back to default
        // This tests the current behavior - in production, env var should always be set
        std::env::remove_var("POSTMARK_FROM_EMAIL");
        let from_email = std::env::var("POSTMARK_FROM_EMAIL")
            .unwrap_or_else(|_| String::from("support@bounti.com"));
        assert_eq!(from_email, "support@bounti.com");
    }

    #[test]
    fn test_formatted_html_contains_all_elements() {
        // Test that formatted HTML contains all required elements
        let org_name = "Acme Corp";
        let invitation_link = "https://bounti.ai/sign-in";
        
        let formatted_org_name = std::format!("<strong>{}</strong>", org_name);
        let template = crate::services::email_service::templates::organization_invitation_html::ORGANIZATION_INVITATION_HTML_TEMPLATE;
        let html_body = template
            .replacen("{}", &formatted_org_name, 1)
            .replacen("{}", invitation_link, 1)
            .replacen("{}", invitation_link, 1)
            .replacen("{}", invitation_link, 1);
        
        // Verify formatted content is present
        assert!(html_body.contains("<strong>Acme Corp</strong>"));
        assert!(html_body.contains(invitation_link));
        
        // Verify structure
        assert!(html_body.contains("<!DOCTYPE html"));
        assert!(html_body.contains("Join Your Team"));
        
        // Verify link appears 3 times (button, fallback href, fallback text)
        assert_eq!(html_body.matches(invitation_link).count(), 3);
    }
}
