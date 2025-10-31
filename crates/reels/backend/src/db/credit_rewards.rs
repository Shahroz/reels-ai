//! Credit reward database models and operations
//!
//! This module provides database models for credit reward entities
//! that map to the credit_reward_definitions and user_credit_rewards tables.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Credit reward definition record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreditRewardDefinition {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "upload_assets")]
    pub action_type: String,
    
    #[schema(example = "Upload Assets Reward")]
    pub action_name: String,
    
    #[schema(example = "Get 10 free credits for uploading 10 assets")]
    pub action_description: String,
    
    #[schema(example = "10")]
    pub required_count: i32,
    
    #[schema(example = "10")]
    pub credit_reward: i32,
    
    #[schema(example = "true")]
    pub is_active: bool,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// User credit reward tracking record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreditReward {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub reward_definition_id: Uuid,
    
    #[schema(example = "5")]
    pub current_count: i32,
    
    #[schema(value_type = Option<String>, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub claimed_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Database row structure for credit reward definitions
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbCreditRewardDefinition {
    pub id: Uuid,
    pub action_type: String,
    pub action_name: String,
    pub action_description: String,
    pub required_count: i32,
    pub credit_reward: i32,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Database row structure for user credit rewards
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUserCreditReward {
    pub id: Uuid,
    pub user_id: Uuid,
    pub reward_definition_id: Uuid,
    pub current_count: i32,
    pub claimed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Database row structure for user credit reward claims
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUserCreditRewardClaim {
    pub id: Uuid,
    pub user_id: Uuid,
    pub reward_definition_id: Uuid,
    pub credits_awarded: i32,
    pub previous_balance: i32,
    pub new_balance: i32,
    pub claimed_at: DateTime<Utc>,
}

impl From<DbCreditRewardDefinition> for CreditRewardDefinition {
    fn from(db: DbCreditRewardDefinition) -> Self {
        Self {
            id: db.id,
            action_type: db.action_type,
            action_name: db.action_name,
            action_description: db.action_description,
            required_count: db.required_count,
            credit_reward: db.credit_reward,
            is_active: db.is_active,
            created_at: db.created_at.unwrap_or_else(|| Utc::now()),
            updated_at: db.updated_at.unwrap_or_else(|| Utc::now()),
        }
    }
}

impl From<DbUserCreditReward> for UserCreditReward {
    fn from(db: DbUserCreditReward) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            reward_definition_id: db.reward_definition_id,
            current_count: db.current_count,
            claimed_at: db.claimed_at,
            created_at: db.created_at.unwrap_or_else(|| Utc::now()),
            updated_at: db.updated_at.unwrap_or_else(|| Utc::now()),
        }
    }
}

/// Combined view of user credit reward with definition details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreditRewardWithDefinition {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub reward_definition_id: Uuid,
    
    #[schema(example = "upload_assets")]
    pub action_type: String,
    
    #[schema(example = "Upload Assets Reward")]
    pub action_name: String,
    
    #[schema(example = "Get 10 free credits for uploading 10 assets")]
    pub action_description: String,
    
    #[schema(example = "10")]
    pub required_count: i32,
    
    #[schema(example = "10")]
    pub credit_reward: i32,
    
    #[schema(example = "5")]
    pub current_count: i32,
    
    #[schema(value_type = Option<String>, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub claimed_at: Option<DateTime<Utc>>,
    
    #[schema(example = "true")]
    pub is_active: bool,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}
