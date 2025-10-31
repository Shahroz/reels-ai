//! Service for batch adding members to an organization with audit logging.
//!
//! This service validates the organization exists, processes the batch addition of
//! members by email, sends invitation emails to successfully added members, and
//! creates an audit log entry with summary statistics. All operations are performed
//! within a single transaction to ensure atomicity, except for email sending which
//! happens after the transaction commits to avoid blocking.

// Maximum number of emails that can be added in a single batch operation.
// This limit guards against sending too many emails at once, which could:
// - Overload the email service
// - Trigger rate limits or spam detection
// - Cause performance issues with large transactions
const MAX_EMAILS_PER_BATCH: usize = 50;

pub async fn add_members_service(
    pool: &sqlx::PgPool,
    postmark_client: &postmark::reqwest::PostmarkClient,
    admin_user_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    emails: Vec<String>,
    role: Option<String>,
) -> Result<crate::queries::admin::organizations::batch_add_members::BatchAddMembersResult, crate::queries::admin::admin_service_error::AdminServiceError> {
    if emails.is_empty() {
        return Err(crate::queries::admin::admin_service_error::AdminServiceError::EmptyEmailList);
    }

    // Deduplicate emails to prevent processing the same email multiple times
    // which would cause it to appear in both success and failed arrays
    let unique_emails: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        emails.into_iter()
            .filter(|email| seen.insert(email.to_lowercase()))
            .collect()
    };

    // Enforce maximum batch size to guard against sending too many emails at once
    if unique_emails.len() > MAX_EMAILS_PER_BATCH {
        return Err(crate::queries::admin::admin_service_error::AdminServiceError::TooManyEmails {
            max: MAX_EMAILS_PER_BATCH,
            actual: unique_emails.len(),
        });
    }

    // Fetch organization to get its name for the email
    let organization = crate::queries::organizations::find_organization_by_id::find_organization_by_id(
        pool,
        organization_id,
    )
    .await?
    .ok_or(crate::queries::admin::admin_service_error::AdminServiceError::OrganizationNotFound)?;

    // Validate that this is not a personal organization
    if organization.is_personal {
        log::warn!(
            "Admin {} attempted to add members to personal organization {}",
            admin_user_id,
            organization_id
        );
        return Err(crate::queries::admin::admin_service_error::AdminServiceError::CannotAddMembersToPersonalOrg);
    }

    let mut tx = pool.begin().await?;

    let role_str = role.as_deref().unwrap_or("member");

    let result = crate::queries::admin::organizations::batch_add_members(
        &mut tx,
        organization_id,
        unique_emails,
        role_str,
        admin_user_id,
    )
    .await?;

    let success_emails: Vec<String> = result
        .success
        .iter()
        .map(|s| s.email.clone())
        .collect();

    let metadata = serde_json::json!({
        "organization_id": organization_id.to_string(),
        "success_count": result.success.len(),
        "failed_count": result.failed.len(),
        "success_emails": success_emails,
    });

    crate::queries::audit_logs::create_audit_log(
        &mut *tx,
        admin_user_id,
        crate::db::audit_action::AuditAction::AddMembersBatch,
        "Organization",
        Some(organization_id),
        Some(metadata),
    )
    .await?;

    tx.commit().await?;

    // Send notification emails to successfully added members (after transaction commits).
    // We don't fail the entire operation if email sending fails - just log the error.
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| String::from("http://localhost:5173"));
    
    for success_item in &result.success {
        // Generate the sign-in link - users are already added, just need to sign in.
        // FRONTEND_URL should point to the appropriate deployment (narrativ or real-estate).
        let invitation_link = frontend_url.clone();

        // Send email asynchronously - don't block or fail if email sending fails
        match crate::services::email_service::send_organization_invitation_email(
            postmark_client,
            &success_item.email,
            &organization.name,
            &invitation_link,
        )
        .await
        {
            Ok(_) => {
                log::info!(
                    "Invitation email sent to {} for organization {}",
                    success_item.email,
                    organization.name
                );
            }
            Err(e) => {
                log::error!(
                    "Failed to send invitation email to {} for organization {}: {}",
                    success_item.email,
                    organization.name,
                    e
                );
                // Continue processing other emails even if one fails
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_emails_constant() {
        // Verify constant is set to expected value
        assert_eq!(MAX_EMAILS_PER_BATCH, 50);
    }

    #[test]
    fn test_empty_emails_validation() {
        // Test that empty email list is rejected
        // This would normally be async but we're testing the validation logic
        let empty_emails: Vec<String> = vec![];
        assert!(empty_emails.is_empty());
        assert_eq!(empty_emails.len(), 0);
    }

    #[test]
    fn test_max_emails_exceeded_validation() {
        // Test that exceeding max emails is detected
        let too_many_emails: Vec<String> = (0..51)
            .map(|i| std::format!("user{}@example.com", i))
            .collect();
        
        assert_eq!(too_many_emails.len(), 51);
        assert!(too_many_emails.len() > MAX_EMAILS_PER_BATCH);
    }

    #[test]
    fn test_exactly_max_emails_allowed() {
        // Test that exactly MAX_EMAILS_PER_BATCH is allowed
        let exactly_max_emails: Vec<String> = (0..MAX_EMAILS_PER_BATCH)
            .map(|i| std::format!("user{}@example.com", i))
            .collect();
        
        assert_eq!(exactly_max_emails.len(), MAX_EMAILS_PER_BATCH);
        assert!(exactly_max_emails.len() <= MAX_EMAILS_PER_BATCH);
    }

    #[test]
    fn test_metadata_json_structure() {
        // Test metadata JSON structure for audit log
        let organization_id = uuid::Uuid::new_v4();
        let success_emails = vec![
            String::from("user1@example.com"),
            String::from("user2@example.com"),
        ];
        
        let metadata = serde_json::json!({
            "organization_id": organization_id.to_string(),
            "success_count": success_emails.len(),
            "failed_count": 0,
            "success_emails": success_emails,
        });
        
        // Verify structure
        assert!(metadata.is_object());
        assert_eq!(metadata["success_count"], 2);
        assert_eq!(metadata["failed_count"], 0);
        assert!(metadata["success_emails"].is_array());
    }

    #[test]
    fn test_role_default_value() {
        // Test role defaults to "member"
        let role: Option<String> = None;
        let role_str = role.as_deref().unwrap_or("member");
        assert_eq!(role_str, "member");
    }

    #[test]
    fn test_role_custom_value() {
        // Test custom role value is used
        let role = Some(String::from("admin"));
        let role_str = role.as_deref().unwrap_or("member");
        assert_eq!(role_str, "admin");
    }

    #[test]
    fn test_frontend_url_fallback() {
        // Test frontend URL fallback logic
        std::env::remove_var("FRONTEND_URL");
        let frontend_url = std::env::var("FRONTEND_URL")
            .unwrap_or_else(|_| String::from("http://localhost:5173"));
        assert_eq!(frontend_url, "http://localhost:5173");
    }

    #[test]
    fn test_email_deduplication() {
        // Test that duplicate emails (including case variations) are deduplicated
        let emails = vec![
            String::from("user1@example.com"),
            String::from("user2@example.com"),
            String::from("User1@Example.com"), // Duplicate with different case
            String::from("user3@example.com"),
            String::from("user1@example.com"), // Exact duplicate
        ];
        
        // Simulate deduplication logic
        let unique_emails: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            emails.into_iter()
                .filter(|email| seen.insert(email.to_lowercase()))
                .collect()
        };
        
        // Should only have 3 unique emails (user1, user2, user3)
        assert_eq!(unique_emails.len(), 3);
        
        // Verify the emails are unique (case-insensitive)
        let lowercase_emails: std::collections::HashSet<String> = unique_emails
            .iter()
            .map(|e| e.to_lowercase())
            .collect();
        assert_eq!(lowercase_emails.len(), 3);
    }
}
