//! Ensure user has tracking for all active credit reward definitions
//!
//! This query ensures that a user has tracking records for all active credit reward definitions.
//! It will insert any missing reward definitions for the user.

use uuid::Uuid;
use sqlx::{PgPool, Result};
use crate::db::credit_rewards::{UserCreditReward, DbUserCreditReward};
use crate::services::credit_rewards::is_user_eligible_for_credit_rewards;

/// Ensure user has tracking records for all active credit reward definitions
/// 
/// This function will:
/// 1. Verify STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
/// 2. Find all active credit reward definitions
/// 3. Check which ones the user doesn't have tracking for
/// 4. Insert missing tracking records with current_count = 0
/// 5. Return all newly created tracking records
pub async fn ensure_user_reward_tracking(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UserCreditReward>> {
    // Check if user is eligible for credit rewards
    let is_eligible = is_user_eligible_for_credit_rewards(pool, user_id).await?;
    if !is_eligible {
        return Err(sqlx::Error::Protocol(
            "Credit rewards are only available for eligible users".into()
        ));
    }
    
    let mut tx = pool.begin().await?;
    let rewards = sqlx::query_as!(
        DbUserCreditReward,
        r#"
        INSERT INTO user_credit_rewards (user_id, reward_definition_id, current_count)
        SELECT $1, crd.id, 0
        FROM credit_reward_definitions crd
        WHERE crd.is_active = true
        AND NOT EXISTS (
            SELECT 1 FROM user_credit_rewards ucr 
            WHERE ucr.user_id = $1 
            AND ucr.reward_definition_id = crd.id
        )
        RETURNING 
            user_credit_rewards.id,
            user_credit_rewards.user_id,
            user_credit_rewards.reward_definition_id,
            user_credit_rewards.current_count,
            user_credit_rewards.claimed_at,
            user_credit_rewards.created_at,
            user_credit_rewards.updated_at
        "#,
        user_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(rewards.into_iter().map(|r| r.into()).collect())
}
