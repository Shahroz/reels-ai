//! Finds a user's ID by their email address.
//!
//! Queries the `users` table for a user with the given email.
//! Returns an `Option<Uuid>` containing the user's ID if found.
//! This is used to resolve an entity for sharing by email.

use sqlx::PgPool;
use uuid::Uuid;

// Helper struct to fetch only user_id
#[derive(sqlx::FromRow)]
struct UserIdRow {
    id: Uuid,
}

pub async fn find_user_id_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let user_row = sqlx::query_as!(UserIdRow, "SELECT id FROM users WHERE email ILIKE $1", email)
        .fetch_optional(pool)
        .await?;
    Ok(user_row.map(|r| r.id))
}
