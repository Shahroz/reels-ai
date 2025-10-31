//! Organization subscription database models and operations.
//!
//! This module provides database models for organization subscription entities
//! that map to the organization_subscriptions table in the database.
//! Follows the same structure as user subscriptions but for organizations.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::schemas::{user_subscription_schemas::SubscriptionStatus, user_credit_allocation_schemas::StripePlanType};

/// Internal database struct for organization subscriptions (matches database schema exactly)
#[derive(sqlx::FromRow)]
pub struct DbOrganizationSubscription {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub stripe_subscription_id: String,
    pub stripe_product_id: String,
    pub stripe_price_id: String,
    pub stripe_plan_type: String,
    pub credits_per_month: i32,
    pub cost: BigDecimal,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl DbOrganizationSubscription {
    /// Convert database struct to public OrganizationSubscription struct
    pub fn into_organization_subscription(self) -> OrganizationSubscription {
        OrganizationSubscription {
            id: self.id,
            organization_id: self.organization_id,
            stripe_subscription_id: self.stripe_subscription_id,
            stripe_product_id: self.stripe_product_id,
            stripe_price_id: self.stripe_price_id,
            stripe_plan_type: StripePlanType::from_str(&self.stripe_plan_type),
            credits_per_month: self.credits_per_month,
            cost: self.cost,
            status: SubscriptionStatus::from_str(&self.status),
            current_period_start: self.current_period_start,
            current_period_end: self.current_period_end,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Organization subscription record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrganizationSubscription {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub organization_id: Uuid,
    
    #[schema(example = "sub_test_subscription_123")]
    pub stripe_subscription_id: String,
    
    #[schema(example = "prod_test_product_123")]
    pub stripe_product_id: String,
    
    #[schema(example = "price_test_price_123")]
    pub stripe_price_id: String,
    
    #[schema(example = "pro")]
    pub stripe_plan_type: StripePlanType,
    
    #[schema(example = "1000")]
    pub credits_per_month: i32,
    
    #[schema(value_type = String, example = "99.99")]
    pub cost: BigDecimal,
    
    #[schema(example = "active")]
    pub status: SubscriptionStatus,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub current_period_start: DateTime<Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub current_period_end: DateTime<Utc>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: Option<DateTime<Utc>>,
}

