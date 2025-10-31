#![allow(clippy::disallowed_methods)]
//! Delete user subscription.
//!
//! This function deletes a user subscription record.

use sqlx::{PgPool, Error};
use tracing::instrument;

/// Delete user subscription
#[instrument(skip(pool))]
pub async fn delete_user_subscription(pool: &PgPool, stripe_subscription_id: &str) -> Result<u64, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM user_subscriptions 
        WHERE stripe_subscription_id = $1
        "#,
        stripe_subscription_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
