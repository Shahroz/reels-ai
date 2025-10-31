//! Increments the token_version for a user to invalidate magic links.
//!
//! This function increments the token_version column using compare-and-swap
//! to prevent race conditions. It only succeeds if the current token_version
//! matches the expected value, ensuring atomic single-use enforcement.

/// Increments the token_version for a user with compare-and-swap.
///
/// # Arguments
///
/// * `pool` - The database connection pool
/// * `user_id` - The UUID of the user whose token_version should be incremented
/// * `expected_version` - The token_version value that must match for update to succeed
///
/// # Returns
///
/// A `Result` indicating success or an `sqlx::Error` on failure.
/// Returns `sqlx::Error::RowNotFound` if the user does not exist or if the
/// token_version doesn't match (indicating the token was already used).
///
/// # Race Condition Prevention
///
/// This function uses compare-and-swap to prevent race conditions where two
/// concurrent requests with the same magic link token could both succeed.
/// Only one request will successfully increment the version.
#[tracing::instrument(skip(pool))]
pub async fn increment_token_version(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    expected_version: i32,
) -> std::result::Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET token_version = token_version + 1, updated_at = NOW()
        WHERE id = $1 AND token_version = $2
        "#,
        user_id,
        expected_version
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return std::result::Result::Err(sqlx::Error::RowNotFound);
    }

    std::result::Result::Ok(())
}

// Note: This query function requires integration tests with a real database.
// Unit tests are not meaningful for database operations.
// Integration tests should be added in backend/tests/auth/magic_link_tests.rs

