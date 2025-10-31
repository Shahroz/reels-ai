#![allow(clippy::disallowed_methods)]
//! Delete user subscription by user ID.
//!
//! This function deletes all user subscription records for a specific user ID.
//! Useful for cleaning up test data or removing all subscriptions for a user.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

/// Delete all user subscriptions for a specific user ID
/// 
/// This function removes all subscription records associated with the given user ID.
/// This is useful for cleaning up test data or removing all subscriptions for a user.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - The user ID whose subscriptions should be deleted
/// 
/// # Returns
/// * `Result<u64, Error>` - Number of rows affected (deleted subscriptions)
#[instrument(skip(pool))]
pub async fn delete_user_subscription_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<u64, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM user_subscriptions 
        WHERE user_id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
