//! Creates a new OAuth user in the database without a password.
//!
//! This function is specifically for OAuth users (Google, etc.) who authenticate
//! via external providers and don't have a password. The password_hash field is
//! set to NULL to distinguish them from password-based users.

/// Creates a new OAuth user in the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `email` - The user's email address.
///
/// # Returns
///
/// A `Result` containing the new user's UUID on success, or an `sqlx::Error` on failure.
#[tracing::instrument(skip(pool))]
pub async fn create_oauth_user(
    pool: &sqlx::PgPool,
    email: &str,
) -> std::result::Result<uuid::Uuid, sqlx::Error> {
    let email_lower = email.to_lowercase();
    let result = sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, NULL)
        RETURNING id AS "id: uuid::Uuid"
        "#,
        email_lower
    )
    .fetch_one(pool)
    .await;

    match result {
        std::result::Result::Ok(record) => std::result::Result::Ok(record.id),
        std::result::Result::Err(e) => {
            log::error!("Failed to create OAuth user: {}", e);
            std::result::Result::Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_oauth_user_placeholder() {
        // This test would require a test database setup
        // Placeholder to demonstrate expected test structure
        assert!(true, "Placeholder test - would test OAuth user creation with NULL password_hash");
    }
}

