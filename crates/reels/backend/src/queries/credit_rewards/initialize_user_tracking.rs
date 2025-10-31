//! Initialize user credit rewards tracking
//!
//! This query creates tracking records for all active reward definitions for a new user.

use crate::db::credit_rewards::{UserCreditReward, DbUserCreditReward};
use sqlx::{PgPool, Result};
use uuid::Uuid;

/// Initialize credit reward tracking for a new user
/// Creates tracking records for all active reward definitions
pub async fn initialize_user_reward_tracking(pool: &PgPool, user_id: Uuid) -> Result<Vec<UserCreditReward>> {
    // Verify STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
    let product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE")
        .unwrap_or_else(|_| "unknown".to_string());
    
    if product_type != "real_estate" {
        return Err(sqlx::Error::Protocol(
            "Credit rewards are only available in Real Estate app".into()
        ));
    }
    
    let mut tx = pool.begin().await?;
    let rewards = sqlx::query_as!(
        DbUserCreditReward,
        r#"
        INSERT INTO user_credit_rewards (user_id, reward_definition_id, current_count)
        SELECT 
            $1,
            crd.id,
            0
        FROM credit_reward_definitions crd
        WHERE crd.is_active = true
        RETURNING 
            id,
            user_id,
            reward_definition_id,
            current_count,
            claimed_at,
            created_at,
            updated_at
        "#,
        user_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(rewards.into_iter().map(|r| r.into()).collect())
}
