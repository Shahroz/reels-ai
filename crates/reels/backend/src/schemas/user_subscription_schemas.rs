use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::schemas::user_credit_allocation_schemas::StripePlanType;

/// Subscription status enum - controlled from code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Canceled,
    PastDue,
    Unpaid,
    Trialing,
    Incomplete,
    IncompleteExpired,
    Paused,
    Trial,
    Expired,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl SubscriptionStatus {
    /// Convert to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Canceled => "canceled",
            SubscriptionStatus::PastDue => "past_due",
            SubscriptionStatus::Unpaid => "unpaid",
            SubscriptionStatus::Trialing => "trialing",
            SubscriptionStatus::Trial => "trial",
            SubscriptionStatus::Incomplete => "incomplete",
            SubscriptionStatus::IncompleteExpired => "incomplete_expired",
            SubscriptionStatus::Paused => "paused",
            SubscriptionStatus::Expired => "expired",
        }
    }

    /// Parse from string (with fallback to Active)
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => SubscriptionStatus::Active,
            "canceled" => SubscriptionStatus::Canceled,
            "past_due" => SubscriptionStatus::PastDue,
            "unpaid" => SubscriptionStatus::Unpaid,
            "trialing" => SubscriptionStatus::Trialing,
            "trial" => SubscriptionStatus::Trial,
            "incomplete" => SubscriptionStatus::Incomplete,
            "incomplete_expired" => SubscriptionStatus::IncompleteExpired,
            "paused" => SubscriptionStatus::Paused,
            "expired" => SubscriptionStatus::Expired,
            _ => SubscriptionStatus::Active, // Default fallback
        }
    }
}

/// Struct for updating user subscription fields
#[derive(Debug, Clone)]
pub struct UserSubscriptionUpdates {
    pub stripe_subscription_id: Option<String>,
    pub stripe_product_id: Option<String>,
    pub stripe_price_id: Option<String>,
    pub stripe_plan_id: Option<String>,
    pub stripe_plan_type: Option<StripePlanType>,
    pub credits: Option<i32>,
    pub cost: Option<BigDecimal>,
    pub status: Option<SubscriptionStatus>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
}

impl Default for UserSubscriptionUpdates {
    fn default() -> Self {
        Self::new()
    }
}

impl UserSubscriptionUpdates {
    /// Create a new empty updates struct
    pub fn new() -> Self {
        Self {
            stripe_subscription_id: None,
            stripe_product_id: None,
            stripe_price_id: None,
            stripe_plan_id: None,
            stripe_plan_type: None,
            credits: None,
            cost: None,
            status: None,
            current_period_start: None,
            current_period_end: None,
        }
    }
    
    /// Builder method to set stripe_subscription_id
    pub fn with_stripe_subscription_id(mut self, value: String) -> Self {
        self.stripe_subscription_id = Some(value);
        self
    }
    
    /// Builder method to set stripe_product_id
    pub fn with_stripe_product_id(mut self, value: String) -> Self {
        self.stripe_product_id = Some(value);
        self
    }
    
    /// Builder method to set stripe_price_id
    pub fn with_stripe_price_id(mut self, value: String) -> Self {
        self.stripe_price_id = Some(value);
        self
    }
    
    /// Builder method to set stripe_plan_id
    pub fn with_stripe_plan_id(mut self, value: String) -> Self {
        self.stripe_plan_id = Some(value);
        self
    }
    
    /// Builder method to set stripe_plan_type
    pub fn with_stripe_plan_type(mut self, value: StripePlanType) -> Self {
        self.stripe_plan_type = Some(value);
        self
    }
    
    /// Builder method to set credits
    pub fn with_credits(mut self, value: i32) -> Self {
        self.credits = Some(value);
        self
    }
    
    /// Builder method to set cost
    pub fn with_cost(mut self, value: BigDecimal) -> Self {
        self.cost = Some(value);
        self
    }
    
    /// Builder method to set status
    pub fn with_status(mut self, value: SubscriptionStatus) -> Self {
        self.status = Some(value);
        self
    }
    
    /// Builder method to set current_period_start
    pub fn with_current_period_start(mut self, value: DateTime<Utc>) -> Self {
        self.current_period_start = Some(value);
        self
    }
    
    /// Builder method to set current_period_end
    pub fn with_current_period_end(mut self, value: DateTime<Utc>) -> Self {
        self.current_period_end = Some(value);
        self
    }
}