//! Organization credit allocation database models and operations.
//!
//! This module provides database models for organization credit allocation entities
//! that map to the organization_credit_allocation table in the database.
//! Tracks the live, spendable credit balance for each organization.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Organization credit allocation record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct OrganizationCreditAllocation {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub organization_id: Uuid,
    
    #[schema(example = "1000.50", value_type = String)]
    pub credits_remaining: BigDecimal,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub last_reset_date: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: Option<DateTime<Utc>>,
}

