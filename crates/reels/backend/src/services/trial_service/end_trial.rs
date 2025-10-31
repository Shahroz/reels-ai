//! Administrative function to manually terminate a user's trial period.
//!
//! This function provides administrative capability to immediately end a user's trial
//! by setting trial_ended_at to the current timestamp and updating subscription_status
//! to expired. Used for administrative actions, abuse prevention, or policy enforcement.
//! Should be used with appropriate authorization checks in calling code.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool))]
pub async fn end_trial(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> std::result::Result<(), sqlx::Error> {
    crate::queries::trial_service::users::end_user_trial(pool, user_id).await
}
