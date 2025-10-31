//! Function to activate a user's subscription after successful payment.
//!
//! This function updates a user's subscription_status to 'active' following successful
//! Stripe payment processing. Called by webhook handlers and payment completion flows
//! to transition users from trial to paid subscription status. This change enables
//! the user to access all premium features and, if they own organizations, extends
//! access to their organization members.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during trial service file splitting
//! - [Prior updates not documented in original file]

#[tracing::instrument(skip(pool))]
pub async fn activate_subscription(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> std::result::Result<(), sqlx::Error> {
    crate::queries::trial_service::users::activate_user_subscription_simple(pool, user_id).await
}
