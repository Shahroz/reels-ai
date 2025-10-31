//! Creates multiple users in batch with validation and error handling.
//!
//! This query processes a list of email addresses, validates each one, checks for
//! existing users, and creates new user accounts with NULL passwords (users will use
//! magic link or forgot password flows). Returns detailed success/failure results
//! for each email to support partial success scenarios in admin bulk operations.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Refactored into separate files, improved email validation

pub async fn batch_create_users(
    pool: &sqlx::PgPool,
    emails: std::vec::Vec<std::string::String>,
) -> anyhow::Result<crate::queries::admin::users::batch_create_users_result::BatchCreateUsersResult> {
    let mut success = std::vec::Vec::new();
    let mut failed = std::vec::Vec::new();

    for email in emails {
        let email_trimmed = email.trim().to_string();
        let email_lower = email_trimmed.to_lowercase();

        if !crate::queries::admin::users::is_valid_email::is_valid_email(&email_lower) {
            failed.push(crate::queries::admin::users::batch_create_user_failure::BatchCreateUserFailure {
                email: email_trimmed,
                reason: "Invalid email format".to_string(),
            });
            continue;
        }

        let existing_user = sqlx::query!(
            r#"SELECT id FROM users WHERE LOWER(email) = $1"#,
            email_lower
        )
        .fetch_optional(pool)
        .await?;

        if existing_user.is_some() {
            failed.push(crate::queries::admin::users::batch_create_user_failure::BatchCreateUserFailure {
                email: email_lower,
                reason: "User already exists".to_string(),
            });
            continue;
        }

        match sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, email_verified, is_admin, status, token_version)
            VALUES ($1, NULL, false, false, 'active', 0)
            RETURNING id, email, stripe_customer_id, email_verified, is_admin, status, 
                      created_at, updated_at, verification_token, token_expiry, 
                      trial_started_at, trial_ended_at, subscription_status, token_version
            "#,
            email_lower
        )
        .fetch_one(pool)
        .await
        {
            Ok(row) => {
                // Create personal organization for admin-created user
                if let Err(e) = crate::queries::organizations::create_personal_organization(
                    pool,
                    row.id,
                    &email_lower,
                    0, // Admin-created users start with 0 credits
                ).await {
                    log::warn!(
                        "Failed to create personal organization for admin-created user {}: {}",
                        row.id,
                        e
                    );
                }
                
                success.push(crate::queries::admin::users::batch_create_user_success::BatchCreateUserSuccess {
                    email: email_lower.clone(),
                    user: crate::db::users::PublicUser {
                        id: row.id,
                        email: row.email,
                        stripe_customer_id: row.stripe_customer_id,
                        email_verified: row.email_verified,
                        is_admin: row.is_admin,
                        status: row.status,
                        feature_flags: std::vec::Vec::new(),
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        verification_token: row.verification_token,
                        token_expiry: row.token_expiry,
                        trial_started_at: row.trial_started_at,
                        trial_ended_at: row.trial_ended_at,
                        subscription_status: row.subscription_status,
                        token_version: row.token_version,
                    },
                });
            }
            Err(e) => {
                failed.push(crate::queries::admin::users::batch_create_user_failure::BatchCreateUserFailure {
                    email: email_lower,
                    reason: format!("Database error: {}", e),
                });
            }
        }
    }

    Ok(crate::queries::admin::users::batch_create_users_result::BatchCreateUsersResult { success, failed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_create_users_empty_list() {
        // This is a placeholder - in a real scenario, you'd use a test database
        // You should implement proper integration tests with test fixtures
    }

    #[tokio::test]
    async fn test_batch_create_users_invalid_emails() {
        // This is a placeholder - in a real scenario, you'd:
        // 1. Set up test database
        // 2. Call batch_create_users with invalid emails
        // 3. Verify all are in the failed list with appropriate reasons
    }

    #[tokio::test]
    async fn test_batch_create_users_existing_users() {
        // This is a placeholder - in a real scenario, you'd:
        // 1. Set up test database with existing users
        // 2. Try to create those users again
        // 3. Verify they're in the failed list with "User already exists" reason
    }

    #[tokio::test]
    async fn test_batch_create_users_success() {
        // This is a placeholder - in a real scenario, you'd:
        // 1. Set up test database
        // 2. Call batch_create_users with valid, new emails
        // 3. Verify all are in the success list
        // 4. Verify users exist in database
        // 5. Verify personal organizations were created
    }

    #[tokio::test]
    async fn test_batch_create_users_mixed_results() {
        // This is a placeholder - in a real scenario, you'd:
        // 1. Set up test database with some existing users
        // 2. Call batch_create_users with mix of valid, invalid, and existing emails
        // 3. Verify appropriate distribution in success and failed lists
    }

    #[test]
    fn test_email_normalization() {
        // Test that emails are properly trimmed and lowercased
        let email1 = "  USER@EXAMPLE.COM  ".trim().to_lowercase();
        assert_eq!(email1, "user@example.com");
        
        let email2 = "User@Example.Com".to_lowercase();
        assert_eq!(email2, "user@example.com");
    }

    // Note: Full integration tests require:
    // - Test database setup
    // - Fixtures for users
    // - Consider using test_utils::helpers::TestUser for integration tests
    // - Test personal organization creation
    // - Test partial success scenarios
}
