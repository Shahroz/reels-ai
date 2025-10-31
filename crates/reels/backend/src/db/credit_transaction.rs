//! Credit transaction database models and operations
//!
//! This module provides database models for credit transaction entities
//! that map to the credit_transactions table in the database.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use bigdecimal::BigDecimal;

/// Credit transaction record for logging credit consumption
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreditTransaction {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub organization_id: Option<Uuid>,
    
    #[schema(example = "5.50", value_type = String)]
    pub credits_changed: BigDecimal,
    
    #[schema(example = "100.25", value_type = String)]
    pub previous_balance: BigDecimal,
    
    #[schema(example = "94.75", value_type = String)]
    pub new_balance: BigDecimal,
    
    #[schema(example = "api")]
    pub action_source: String,
    
    #[schema(example = "retouch_image")]
    pub action_type: String,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub entity_id: Option<Uuid>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Database row structure for credit transactions
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbCreditTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub credits_changed: BigDecimal,
    pub previous_balance: BigDecimal,
    pub new_balance: BigDecimal,
    pub action_source: String,
    pub action_type: String,
    pub entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<DbCreditTransaction> for CreditTransaction {
    fn from(db_row: DbCreditTransaction) -> Self {
        Self {
            id: db_row.id,
            user_id: db_row.user_id,
            organization_id: db_row.organization_id,
            credits_changed: db_row.credits_changed,
            previous_balance: db_row.previous_balance,
            new_balance: db_row.new_balance,
            action_source: db_row.action_source,
            action_type: db_row.action_type,
            entity_id: db_row.entity_id,
            created_at: db_row.created_at,
        }
    }
}
