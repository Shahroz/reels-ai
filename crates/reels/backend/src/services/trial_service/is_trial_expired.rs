//! Function to check if a user's trial period has expired.
//!
//! This function performs a simple boolean check to determine trial expiration status
//! by examining trial_started_at and trial_ended_at timestamps. Used for administrative
//! purposes and legacy compatibility. New code should use get_trial_status instead
//! which provides more comprehensive trial state information.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool))]
pub async fn is_trial_expired(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> std::result::Result<bool, sqlx::Error> {
    let user_trial_info = crate::queries::trial_service::users::get_user_trial_info(pool, user_id).await?;

    if let std::option::Option::Some(trial_ended_at) = user_trial_info.trial_ended_at {
        return std::result::Result::Ok(chrono::Utc::now() > trial_ended_at);
    }

    if let std::option::Option::Some(trial_started_at) = user_trial_info.trial_started_at {
        let trial_end_date = trial_started_at + chrono::Duration::days(crate::services::trial_service::get_trial_period_days::get_trial_period_days());
        std::result::Result::Ok(chrono::Utc::now() > trial_end_date)
    } else {
        std::result::Result::Ok(false)
    }
}
