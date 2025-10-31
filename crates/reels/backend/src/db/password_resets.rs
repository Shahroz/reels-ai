// backend/src/db/password_resets.rs
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPool, types::Uuid, Error};
use tracing::instrument;

/// Stores a password reset token in the database.
///
/// If a token already exists for the user, it will be updated.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user requesting the reset.
/// * `token` - The generated password reset token.
/// * `expires_at` - The expiry time for the token.
///
/// # Returns
///
/// A `Result` indicating success or an `sqlx::Error` on failure.
#[instrument(skip(pool, token))]
pub async fn store_reset_token(
   pool: &PgPool,
   user_id: Uuid,
   token: &str,
   expires_at: DateTime<Utc>,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    // First, delete any existing reset tokens for this user to ensure they can only have one active at a time.
    sqlx::query!(
        r#"
        DELETE FROM password_reset_tokens
        WHERE user_id = $1
        "#,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    // Now, insert the new token.
    sqlx::query!(
        r#"
        INSERT INTO password_reset_tokens (token, user_id, expires_at)
        VALUES ($1, $2, $3)
        "#,
        token,
        user_id,
        expires_at
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Finds a user ID and token expiry by the password reset token.
/// Also checks if the token has expired.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `token` - The password reset token to look up.
///
/// # Returns
///
/// A `Result` containing an `Option` with the `(user_id, expires_at)` tuple if a valid,
/// non-expired token is found, or `None` otherwise. Returns `sqlx::Error` on database errors.
#[instrument(skip(pool, token))]
pub async fn find_user_by_reset_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<(Uuid, DateTime<Utc>)>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT user_id AS "user_id: uuid::Uuid", expires_at
        FROM password_reset_tokens
        WHERE token = $1 AND expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| (row.user_id, row.expires_at)))
}

/// Deletes a password reset token from the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `token` - The password reset token to delete.
///
/// # Returns
///
/// A `Result` indicating success or an `sqlx::Error` on failure.
#[instrument(skip(pool, token))]
pub async fn delete_reset_token(pool: &PgPool, token: &str) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM password_reset_tokens
        WHERE token = $1
        "#,
        token
    )
    .execute(pool)
    .await?;

    Ok(())
}
