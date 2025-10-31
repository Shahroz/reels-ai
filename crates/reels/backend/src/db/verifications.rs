use chrono::{DateTime, Utc};
use sqlx::{Error, PgPool};
use uuid::Uuid;
use tracing::instrument;

/// Stores the verification token and expiry date for a user.
///
/// Updates the `users` table, setting the `verification_token` and `token_expiry`
/// columns for the specified `user_id`.
#[instrument(skip(pool, token))]
pub async fn store_verification_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET verification_token = $1, token_expiry = $2
        WHERE id = $3::uuid
        "#,
        token,
        expires_at,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
