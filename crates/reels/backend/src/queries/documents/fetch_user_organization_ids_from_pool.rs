//! Fetches the active organization IDs for a given user using a connection pool.
//!
//! This function queries the `organization_members` table to find all organizations
//! where the user is an 'active' member. It returns a vector of organization UUIDs.
//! This pool-based version is used for permission checks before starting transactions,
//! avoiding connection pool conflicts when a transaction is already in progress.

pub async fn fetch_user_organization_ids_from_pool(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> std::result::Result<std::vec::Vec<uuid::Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .fetch_all(pool)
    .await
}

// Note: This function is a simple wrapper around sqlx::query_scalar!
// Integration tests in the test suite provide actual database validation. 