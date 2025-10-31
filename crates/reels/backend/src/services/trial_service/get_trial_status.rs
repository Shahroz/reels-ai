//! Function to calculate the current trial status for a user.
//!
//! This function determines whether a user's trial is active, expired, or not started
//! by examining trial_started_at and trial_ended_at timestamps in combination with
//! the configured trial period duration. It handles both manual trial termination
//! and automatic expiration based on time elapsed since trial start.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool))]
pub async fn get_trial_status(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> std::result::Result<crate::services::trial_service::trial_status::TrialStatus, sqlx::Error> {
    let config = crate::services::trial_service::trial_config::TrialConfig::from_env();
    crate::services::trial_service::get_trial_status_with_config::get_trial_status_with_config(pool, user_id, &config).await
}
