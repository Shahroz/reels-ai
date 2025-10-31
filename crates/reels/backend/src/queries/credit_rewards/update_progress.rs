//! Update user credit reward progress
//!
//! This query updates the progress count for a user's credit reward based on action type.

use uuid::Uuid;
use sqlx::{PgPool, Result};
use crate::db::credit_rewards::{UserCreditReward, DbUserCreditReward};
use crate::services::credit_rewards::is_user_eligible_for_credit_rewards;

/// Update user credit reward progress for a specific action type
/// 
/// This function will:
/// 1. Try to update existing user reward progress
/// 2. If no record exists, create a new one with the reward count
/// 3. Return the updated/created reward records
pub async fn update_user_reward_progress(
    pool: &PgPool,
    user_id: Uuid,
    action_type: &str,
    reward_count: i32,
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
        SELECT $1, crd.id, LEAST($3, crd.required_count)
        FROM credit_reward_definitions crd
        WHERE crd.action_type = $2 AND crd.is_active = true
        ON CONFLICT (user_id, reward_definition_id) 
        DO UPDATE SET 
            current_count = LEAST(user_credit_rewards.current_count + $3, 
                (SELECT required_count FROM credit_reward_definitions WHERE id = user_credit_rewards.reward_definition_id)),
            updated_at = CURRENT_TIMESTAMP
        WHERE user_credit_rewards.claimed_at IS NULL
        RETURNING 
            user_credit_rewards.id,
            user_credit_rewards.user_id,
            user_credit_rewards.reward_definition_id,
            user_credit_rewards.current_count,
            user_credit_rewards.claimed_at,
            user_credit_rewards.created_at,
            user_credit_rewards.updated_at
        "#,
        user_id,
        action_type,
        reward_count
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(rewards.into_iter().map(|r| r.into()).collect())
}
