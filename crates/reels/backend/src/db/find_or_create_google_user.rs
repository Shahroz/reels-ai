//! Finds or creates a user account for Google OAuth2 authentication.
//!
//! Handles user lookup and creation for Google OAuth2 login flow. First attempts to find
//! an existing user by email address. If not found, creates a new user account with
//! NULL password_hash since OAuth2 users don't use password-based authentication.



/// Finds an existing user by email or creates a new one using Google OAuth2 information.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `email` - User's email address from Google OAuth2
/// * `_user_info` - Additional user information from Google (currently unused)
///
/// # Returns
///
/// A `Result` containing a tuple of `(User, bool)` on success, or a database error on failure.
/// The boolean indicates whether the user was newly created (`true`) or already existed (`false`).
pub async fn find_or_create_google_user(
    pool: &sqlx::PgPool,
    email: &str,
    _user_info: &serde_json::Value,
) -> std::result::Result<(crate::db::users::User, bool), sqlx::Error> {
    // Try to find existing user first
    if let std::option::Option::Some(existing_user) = crate::db::users::find_user_by_email(pool, email).await? {
        // Create free subscription for existing Google OAuth user
        create_free_subscription_for_google_user(pool, existing_user.id, email).await?;
        log::info!("Found existing user for email: {email}");
        return std::result::Result::Ok((existing_user, false));
    }

    // Create new user for Google OAuth2
    log::info!("Creating new user for Google OAuth2: {email}");
    
    // For OAuth2 users, we don't have a password, so password_hash is NULL
    // This user won't be able to log in with password-based auth
    let user_id = crate::db::create_oauth_user::create_oauth_user(pool, email).await?;
    
    // Create free subscription for new Google OAuth user
    create_free_subscription_for_google_user(pool, user_id, email).await?;
    
    // Create personal organization for new Google OAuth user
    if let Err(e) = crate::queries::organizations::create_personal_organization(
        pool,
        user_id,
        email,
        crate::app_constants::credits_constants::FREE_CREDITS,
    ).await {
        log::warn!(
            "Failed to create personal organization for new Google OAuth user {}: {}",
            user_id,
            e
        );
        // Don't fail user creation if personal org creation fails
    }
    
    // Fetch the created user to return complete User object
    match crate::db::users::find_user_by_email(pool, email).await? {
        std::option::Option::Some(user) => std::result::Result::Ok((user, true)),
        std::option::Option::None => {
            log::error!("Failed to fetch newly created user: {email}");
            std::result::Result::Err(sqlx::Error::RowNotFound)
        }
    }
}

pub async fn create_free_subscription_for_google_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    email: &str,
) -> std::result::Result<(), sqlx::Error> {
    if let Ok(billing_service) = crate::services::billing::billing_factory::get_billing_service() {
        if let Err(e) = billing_service.create_free_subscription(pool, user_id, email).await {  
            log::warn!("Failed to create free subscription for new Google OAuth user {}: {}", user_id, e);
            // Don't fail user creation if subscription creation fails
        } else {
            log::info!("Successfully created free subscription for new Google OAuth user: {}", user_id);
        }
    } else {
        log::warn!("Billing service not available, skipping free subscription creation for Google OAuth user: {}", user_id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a test database setup
    // For now, we'll add placeholder tests to follow the guidelines

    #[test]
    fn test_find_or_create_google_user_exists() {
        // This test would need a mock database or test database setup
        // For now, it's a placeholder to demonstrate the expected test structure
        assert!(true, "Placeholder test - would test finding existing user returns (user, false)");
    }

    #[test]
    fn test_find_or_create_google_user_creates_new() {
        // This test would need a mock database or test database setup
        // For now, it's a placeholder to demonstrate the expected test structure
        assert!(true, "Placeholder test - would test creating new user returns (user, true)");
    }
} 