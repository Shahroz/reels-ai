//! Get all active credit reward definitions
//!
//! This query retrieves all active credit reward definitions from the database.

use crate::db::credit_rewards::{CreditRewardDefinition, DbCreditRewardDefinition};
use sqlx::{PgPool, Result};

/// Get all active credit reward definitions
pub async fn get_active_reward_definitions(pool: &PgPool) -> Result<Vec<CreditRewardDefinition>> {
    // Verify STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
    let product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE")
        .unwrap_or_else(|_| "unknown".to_string());
    
    if product_type != "real_estate" {
        return Ok(vec![]);
    }
    
    let definitions = sqlx::query_as!(
        DbCreditRewardDefinition,
        r#"
        SELECT 
            id,
            action_type,
            action_name,
            action_description,
            required_count,
            credit_reward,
            is_active,
            created_at,
            updated_at
        FROM credit_reward_definitions 
        WHERE is_active = true
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(definitions.into_iter().map(|d| d.into()).collect())
}

/// Get a specific reward definition by reward key
pub async fn get_reward_definition_by_key(pool: &PgPool, reward_key: &str) -> Result<Option<CreditRewardDefinition>> {
    // Verify STRIPE_METADATA_PRODUCT_TYPE is set to "real_estate"
    let product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE")
        .unwrap_or_else(|_| "unknown".to_string());
    
    if product_type != "real_estate" {
        return Ok(None);
    }

    let definition = sqlx::query_as!(
        DbCreditRewardDefinition,
        r#"
        SELECT 
            id,
            action_type,
            action_name,
            action_description,
            required_count,
            credit_reward,
            is_active,
            created_at,
            updated_at
        FROM credit_reward_definitions 
        WHERE action_type = $1 AND is_active = true
        "#,
        reward_key
    )
    .fetch_optional(pool)
    .await?;

    Ok(definition.map(|d| d.into()))
}
