//! Claim credit rewards
//!
//! This query handles claiming credit rewards when requirements are met.

use uuid::Uuid;
use sqlx::{PgPool, Result, Row};
use crate::db::credit_rewards::{UserCreditReward, DbUserCreditReward};
use crate::queries::credit_transactions::create_credit_transaction;
use crate::schemas::credit_transactions_schemas::CreateCreditTransactionParams;
use crate::services::credit_rewards::is_user_eligible_for_credit_rewards;
use crate::queries::organizations::get_user_personal_organization::get_user_personal_organization;
use bigdecimal::BigDecimal;

/// Check if a user can claim a specific reward
pub async fn can_claim_reward(pool: &PgPool, user_id: Uuid, reward_definition_id: Uuid) -> Result<Option<UserCreditReward>> {
    let mut tx = pool.begin().await?;
    let reward = sqlx::query_as!(
        DbUserCreditReward,
        r#"
        SELECT 
            ucr.id,
            ucr.user_id,
            ucr.reward_definition_id,
            ucr.current_count,
            ucr.claimed_at,
            ucr.created_at,
            ucr.updated_at
        FROM user_credit_rewards ucr
        JOIN credit_reward_definitions crd ON ucr.reward_definition_id = crd.id
        WHERE 
            ucr.user_id = $1 
            AND ucr.reward_definition_id = $2
            AND crd.is_active = true
            AND ucr.claimed_at IS NULL
            AND ucr.current_count >= crd.required_count
        "#,
        user_id,
        reward_definition_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    Ok(reward.map(|r| r.into()))
}

/// Claim a credit reward for a user
pub async fn claim_credit_reward(
    pool: &PgPool,
    user_id: Uuid,
    reward_definition_id: Uuid,
) -> Result<Option<i32>> {
    // Check if user is eligible for credit rewards
    let is_eligible = is_user_eligible_for_credit_rewards(pool, user_id).await?;
    if !is_eligible {
        return Err(sqlx::Error::Protocol(
            "Credit rewards are only available for eligible users".into()
        ));
    }
    
    let mut tx = pool.begin().await?;

    // First, check if the reward can be claimed and get the credit amount and action type
    let reward_data = sqlx::query(
        r#"
        SELECT 
            ucr.id as user_reward_id,
            crd.credit_reward,
            crd.action_type,
            ucr.current_count
        FROM user_credit_rewards ucr
        JOIN credit_reward_definitions crd ON ucr.reward_definition_id = crd.id
        WHERE 
            ucr.user_id = $1 
            AND ucr.reward_definition_id = $2
            AND crd.is_active = true
            AND ucr.claimed_at IS NULL
            AND ucr.current_count >= crd.required_count
        FOR UPDATE
        "#
    )
    .bind(user_id)
    .bind(reward_definition_id)
    .fetch_optional(&mut *tx)
    .await?;

    let reward_data = match reward_data {
        Some(row) => {
            let user_reward_id: Uuid = row.get("user_reward_id");
            let credit_reward: i32 = row.get("credit_reward");
            let action_type: String = row.get("action_type");
            let current_count: i32 = row.get("current_count");
            (user_reward_id, credit_reward, action_type, current_count)
        },
        None => {
            tx.rollback().await?;
            return Ok(None);
        }
    };

    // Get user's personal organization
    let personal_org = get_user_personal_organization(pool, user_id).await?
        .ok_or_else(|| sqlx::Error::Protocol("User has no personal organization".into()))?;

    // Get current credit balance from organization credit allocation
    let current_balance = sqlx::query!(
        r#"
        SELECT credits_remaining
        FROM organization_credit_allocation
        WHERE organization_id = $1
        "#,
        personal_org.id
    )
    .fetch_optional(&mut *tx)
    .await?
    .map(|row| row.credits_remaining)
    .unwrap_or(BigDecimal::from(0));

    // Add credits to personal organization's balance
    sqlx::query(
        r#"
        UPDATE organization_credit_allocation
        SET 
            credits_remaining = credits_remaining + $1,
            updated_at = CURRENT_TIMESTAMP
        WHERE organization_id = $2
        "#
    )
    .bind(reward_data.1)
    .bind(personal_org.id)
    .execute(&mut *tx)
    .await?;

    // Mark reward as claimed
    sqlx::query(
        r#"
        UPDATE user_credit_rewards 
        SET 
            claimed_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#
    )
    .bind(reward_data.0)
    .execute(&mut *tx)
    .await?;

    // Commit the transaction first
    tx.commit().await?;

    // Create transaction record for audit purposes
    let _transaction_result = create_credit_transaction(
        pool,
        CreateCreditTransactionParams {
            user_id,
            organization_id: Some(personal_org.id), // Credit rewards are now added to personal organization
            credits_changed: BigDecimal::from(reward_data.1), // Positive value indicates credit addition
            previous_balance: current_balance.clone(),
            new_balance: current_balance + BigDecimal::from(reward_data.1),
            action_source: "api".to_string(),
            action_type: format!("claim_reward_{}", reward_data.2), // e.g., "claim_reward_upload_assets"
            entity_id: None, // Not specified for reward claims
        },
    ).await?;

    Ok(Some(reward_data.1))
}
