//! Get user credit rewards with definitions
//!
//! This query retrieves all credit rewards for a user with their definition details.

use crate::db::credit_rewards::UserCreditRewardWithDefinition;
use chrono::Utc;
use sqlx::{PgPool, Result, Row};
use uuid::Uuid;

/// Get all credit rewards for a user with definition details
pub async fn get_user_credit_rewards(pool: &PgPool, user_id: Uuid) -> Result<Vec<UserCreditRewardWithDefinition>> {
    // Verify STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
    let product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE")
        .unwrap_or_else(|_| "unknown".to_string());
    
    if product_type != "real_estate" {
        return Ok(vec![]);
    }
    let rewards = sqlx::query(
        r#"
        SELECT 
            ucr.id,
            ucr.user_id,
            ucr.reward_definition_id,
            crd.action_type,
            crd.action_name,
            crd.action_description,
            crd.required_count,
            crd.credit_reward,
            ucr.current_count,
            ucr.claimed_at,
            crd.is_active,
            ucr.created_at,
            ucr.updated_at
        FROM user_credit_rewards ucr
        JOIN credit_reward_definitions crd ON ucr.reward_definition_id = crd.id
        WHERE ucr.user_id = $1 AND crd.is_active = true
        ORDER BY crd.created_at ASC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rewards.into_iter().map(|row| UserCreditRewardWithDefinition {
        id: row.get("id"),
        user_id: row.get("user_id"),
        reward_definition_id: row.get("reward_definition_id"),
        action_type: row.get("action_type"),
        action_name: row.get("action_name"),
        action_description: row.get("action_description"),
        required_count: row.get("required_count"),
        credit_reward: row.get("credit_reward"),
        current_count: row.get("current_count"),
        claimed_at: row.get("claimed_at"),
        is_active: row.get("is_active"),
        created_at: row.get::<Option<chrono::DateTime<Utc>>, _>("created_at").unwrap_or_else(|| Utc::now()),
        updated_at: row.get::<Option<chrono::DateTime<Utc>>, _>("updated_at").unwrap_or_else(|| Utc::now()),
    }).collect())
}
