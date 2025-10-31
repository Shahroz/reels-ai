//! User credit allocation database models and operations
//!
//! This module provides database models for user credit allocation entities
//! that map to existing tables in the database.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::schemas::user_credit_allocation_schemas::StripePlanType;

/// User credit allocation record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreditAllocation {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    #[schema(example = "pro")]
    pub plan_type: StripePlanType,
    
    #[schema(example = "2")]
    pub daily_credits: i32,
    
    #[schema(example = "30")]
    pub plan_credits: i32,
    
    #[schema(example = "30.50", value_type = String)]
    pub credits_remaining: BigDecimal,
    
    #[schema(example = "10")]
    pub credit_limit: i32,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub last_daily_credit_claimed_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00Z")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: Option<DateTime<Utc>>,
}