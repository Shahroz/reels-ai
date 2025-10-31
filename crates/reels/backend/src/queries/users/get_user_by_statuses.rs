#![allow(clippy::disallowed_methods)]
//! Get user's trial active user.
//!
//! This function retrieves the current active subscription for a user.

use sqlx::{PgPool, Error};
use tracing::instrument;
use uuid::Uuid;

use crate::db::users::User;
use crate::schemas::user_subscription_schemas::SubscriptionStatus;

/// Get user's trial active user
#[instrument(skip(pool))]
pub async fn get_user_by_statuses(pool: &PgPool, user_id: Uuid, statuses: &[SubscriptionStatus]) -> Result<Option<User>, Error> {
    // Convert SubscriptionStatus enum values to strings for SQLx
    let status_strings: Vec<&str> = statuses.iter().map(|status| status.as_str()).collect();

    let results = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, stripe_customer_id, email_verified, is_admin, status, feature_flags,
            created_at, updated_at, verification_token, token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        FROM users
        WHERE id = $1 
          AND subscription_status = ANY($2)
        ORDER BY created_at DESC
        "#,
        user_id,
        &status_strings as &[&str]
    )
    .fetch_optional(pool)
    .await?;

    Ok(results)
}
