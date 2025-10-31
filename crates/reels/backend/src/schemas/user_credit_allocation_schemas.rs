use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::db::user_credit_allocation::{UserCreditAllocation};

/// Stripe plan type enum - controlled from code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StripePlanType {
    Free,
    Pro,
    Annual,
    Unknown,
}

impl std::fmt::Display for StripePlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl StripePlanType {
    /// Convert to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            StripePlanType::Free => "free",
            StripePlanType::Pro => "pro",
            StripePlanType::Annual => "annual",
            StripePlanType::Unknown => "unknown",
        }
    }

    /// Parse from string (with fallback to Free)
    pub fn from_str(s: &str) -> Self {
        match s {
            "free" => StripePlanType::Free,
            "pro" => StripePlanType::Pro,
            "annual" => StripePlanType::Annual,
            _ => StripePlanType::Unknown, // Default fallback
        }
    }

    /// Get the numeric tier value for comparison (higher number = higher tier)
    pub fn tier_value(&self) -> u8 {
        match self {
            StripePlanType::Free => 0,
            StripePlanType::Pro => 1,
            StripePlanType::Annual => 2,
            StripePlanType::Unknown => 0,
        }
    }

    /// Check if this plan is higher tier than another plan
    pub fn is_higher_tier_than(&self, other: &StripePlanType) -> bool {
        self.tier_value() > other.tier_value()
    }

    /// Check if this plan is lower tier than another plan
    pub fn is_lower_tier_than(&self, other: &StripePlanType) -> bool {
        self.tier_value() < other.tier_value()
    }

    /// Check if this plan is the same tier as another plan
    pub fn is_same_tier_as(&self, other: &StripePlanType) -> bool {
        self.tier_value() == other.tier_value()
    }
}

/// Check if the old value credits is the same as the new value credits
pub fn is_same_credits(old_value: BigDecimal, new_value: BigDecimal) -> bool {
    old_value == new_value
}

/// Check if the old value credits is less than the new value credits
pub fn is_less_than_credits(old_value: BigDecimal, new_value: BigDecimal) -> bool {
    old_value < new_value
}

/// Check if the old value credits is greater than the new value credits
pub fn is_greater_than_credits(old_value: BigDecimal, new_value: BigDecimal) -> bool {
    old_value > new_value
}

/// Internal database struct for user credit allocations (matches database schema exactly)
#[derive(sqlx::FromRow)]
pub struct DbUserCreditAllocation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_type: String,
    pub daily_credits: i32,
    pub plan_credits: i32,
    pub credits_remaining: BigDecimal,
    pub credit_limit: i32,
    pub last_daily_credit_claimed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl DbUserCreditAllocation {
    /// Convert database struct to public UserCreditAllocation struct
    pub fn into_user_credit_allocation(self) -> UserCreditAllocation {
        UserCreditAllocation {
            id: self.id,
            user_id: self.user_id,
            plan_type: StripePlanType::from_str(&self.plan_type),
            daily_credits: self.daily_credits,
            plan_credits: self.plan_credits,
            credits_remaining: self.credits_remaining,
            credit_limit: self.credit_limit,
            last_daily_credit_claimed_at: self.last_daily_credit_claimed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}