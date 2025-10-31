//! Fetches a user's email address by their unique user ID using pool connections.
//!
//! This function queries the `users` table for a single user's email using
//! pool connections. It's used when an operation needs to associate an action
//! with a user's email, such as populating creator information on a newly
//! created document. This pool-based version is used for read-only operations
//! before starting transactions, optimizing connection usage and performance.

/// Fetches a user's email address using pool connections.
///
/// This function queries the `users` table for a single user's email.
/// It uses pool connections for read-only operations before any transactions
/// are started, optimizing connection usage and performance for document operations.
pub async fn fetch_user_email_from_pool(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> std::result::Result<std::option::Option<std::string::String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT email FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await
}

// Note: This function is a simple wrapper around sqlx::query_scalar!
// User email fetching is tested in the integration test suite. 