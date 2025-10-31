use crate::auth::tokens::{generate_password_reset_token, generate_verification_token};
use crate::db::password_resets;
use crate::db::users::{self, User}; // Import User struct
use crate::email_service;
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;
use crate::utils::password_validator::validate_password;

// Removed unused: use chrono::{Utc, Duration};

#[instrument(skip(pool, postmark_client, email, password_hash))]
pub async fn register_user(
    pool: &PgPool,
    postmark_client: &postmark::reqwest::PostmarkClient,
    email: &str,
    password_hash: &str,
    send_verification_token: bool,
) -> Result<Uuid> {
    // Create user record
    let user_id = users::create_user(pool, email, password_hash).await?;
    // Generate email verification token
    if send_verification_token {
        let (token, _expires_at) = generate_verification_token(); // Used tokens::, prefixed expires_at
        // TODO: Here, you would update the user record with the token and expiry
        // Send verification email
        email_service::send_verification_email(postmark_client, user_id, email, &token)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
    }
    Ok(user_id)
}

// Function to find user by email, needed by auth::login
#[instrument(skip(pool, email))]
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    users::find_user_by_email(pool, email)
        .await
        .map_err(|e| anyhow::anyhow!("Database error finding user by email: {}", e))
}

// Placeholder login logic - actual password verification happens in auth::login
#[instrument(skip(pool, email, _password))]
pub async fn login_user(pool: &PgPool, email: &str, _password: &str) -> Result<Uuid> {
    // This function is now less useful as password check is in auth::login
    // It primarily serves to check if the user exists by email.
    let user_option = find_user_by_email(pool, email).await?;
    if let Some(user) = user_option {
        Ok(user.id)
    } else {
        Err(anyhow::anyhow!("User not found")) // Error message refined
    }
}

#[instrument(skip(pool, email))]
pub async fn initiate_password_reset(
    pool: &PgPool,
    postmark_client: &postmark::reqwest::PostmarkClient,
    email: &str,
) -> Result<()> {
    // Lookup user by email
    let user_option = find_user_by_email(pool, email).await?;
    if let Some(user) = user_option {
        // Generate a password reset token valid for 1 hour
        let (token, expires_at) = generate_password_reset_token(); // Used tokens::
        password_resets::store_reset_token(pool, user.id, &token, expires_at).await?;
        // Send password reset email
        email_service::send_password_reset_email(postmark_client, user.id, email, &token).await?;
        Ok(())
    } else {
        // Don't signal if email not found for security
        log::info!("Password reset requested for non-existent email: {email}");
        Ok(())
    }
}

#[instrument(skip(pool))]
pub async fn find_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<crate::db::users::User>, sqlx::Error> {
    users::find_user_by_id(pool, user_id).await
}

#[instrument(skip(pool, token, new_password))]
pub async fn reset_password(pool: &PgPool, token: &str, new_password: &str) -> Result<()> {
    // 1. Validate the token and get user_id
    let (user_id, _expires_at) = password_resets::find_user_by_reset_token(pool, token)
        .await?
        .ok_or_else(|| anyhow!("Invalid or expired password reset token"))?;

    // 2. Enforce password policy before hashing
    if let Err(msg) = validate_password(new_password) {
        return Err(anyhow!(msg));
    }
    // 3. Hash the new password
    let new_password_hash =
        bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|e| anyhow!(e))?;

    // 3. Update the user's password
    users::update_user_password_hash(pool, user_id, &new_password_hash).await?;

    // 4. Delete the used reset token
   password_resets::delete_reset_token(pool, token).await?;

   Ok(())
}

/// Changes a user's password after verifying their current one.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user changing their password.
/// * `current_password` - The user's current password for verification.
/// * `new_password` - The new password to set.
///
/// # Returns
///
/// An `anyhow::Result<()>` indicating success or failure.
#[instrument(skip(pool, current_password, new_password))]
pub async fn change_user_password(
    pool: &PgPool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<()> {
    // 1. Find the user by ID
    let user = find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;

    // 2. Verify the current password
    let password_hash = user.password_hash
        .as_ref()
        .ok_or_else(|| anyhow!("This account uses OAuth authentication and doesn't have a password"))?;
    
    let valid_password = bcrypt::verify(current_password, password_hash)
        .map_err(|e| anyhow!("Error during password verification: {}", e))?;
    if !valid_password {
        return Err(anyhow!("Incorrect current password"));
    }

    // 3. Validate the new password
    if let Err(msg) = validate_password(new_password) {
        return Err(anyhow!(msg));
    }

    // 4. Hash the new password
    let new_password_hash =
        bcrypt::hash(new_password, bcrypt::DEFAULT_COST).map_err(|e| anyhow!(e))?;

    // 5. Update the password in the database
    users::update_user_password_hash(pool, user_id, &new_password_hash).await?;

    Ok(())
}
