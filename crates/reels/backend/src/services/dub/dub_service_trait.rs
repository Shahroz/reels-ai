//! Dub service trait for dependency injection
//!
//! This trait defines the interface for Dub attribution tracking services,
//! enabling dependency injection and testability following the project's patterns.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lead event data for Dub attribution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubLeadEvent {
    /// Unique customer identifier (user UUID)
    pub customer_id: String,
    /// Customer email address
    pub customer_email: String,
    /// Event name (e.g., "Sign Up", "Account Created")
    pub event_name: String,
    /// Click ID for attribution (dub_id from URL parameter or cookie)
    pub click_id: Option<String>,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Sale event data for Dub attribution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubSaleEvent {
    /// Unique customer identifier (user UUID)
    pub customer_id: String,
    /// Sale amount in cents
    pub amount_cents: i64,
    /// Currency code (e.g., "USD")
    pub currency: String,
    /// Event name (e.g., "Subscription", "Purchase")
    pub event_name: String,
    /// Click ID for attribution (dub_id from URL parameter or cookie)
    pub click_id: Option<String>,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Response from Dub API for tracking events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubEventResponse {
    /// Indicates if the event was successfully tracked
    pub success: bool,
    /// Event ID assigned by Dub
    pub event_id: Option<String>,
    /// Any error message from Dub
    pub message: Option<String>,
}

/// Trait for Dub attribution tracking service
#[async_trait]
pub trait DubServiceTrait: Send + Sync {
    /// Track a lead event (e.g., user sign-up)
    async fn track_lead_event(&self, event: DubLeadEvent) -> Result<DubEventResponse>;

    /// Track a sale event (e.g., subscription purchase)
    async fn track_sale_event(&self, event: DubSaleEvent) -> Result<DubEventResponse>;

    /// Generate a referral token for embedded referral dashboard
    async fn generate_referral_token(
        &self,
        tenant_id: String,
        partner_name: String,
        partner_email: String,
        partner_image: Option<String>,
        group_id: Option<String>,
    ) -> Result<String>;

    /// Check if Dub tracking is enabled
    fn is_enabled(&self) -> bool;

    /// Get the workspace ID for frontend configuration
    fn get_workspace_id(&self) -> Option<String>;
}

impl DubLeadEvent {
    /// Create a new lead event for user registration
    pub fn new_signup(user_id: Uuid, email: String) -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String("registration".to_string()));
        
        DubLeadEvent {
            customer_id: user_id.to_string(),
            customer_email: email,
            event_name: "Sign Up".to_string(),
            click_id: None, // Will be set later if available
            metadata,
        }
    }

    /// Add metadata to the lead event
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl DubSaleEvent {
    /// Create a new sale event for subscription
    pub fn new_subscription(customer_id: String, amount_cents: i64) -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("product_type".to_string(), serde_json::Value::String("subscription".to_string()));
        
        DubSaleEvent {
            customer_id,
            amount_cents,
            currency: "USD".to_string(),
            event_name: "Subscription".to_string(),
            click_id: None, // Will be set later if available
            metadata,
        }
    }

    /// Add metadata to the sale event
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_event_creation() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        
        let event = DubLeadEvent::new_signup(user_id, email.clone());
        
        assert_eq!(event.customer_id, user_id.to_string());
        assert_eq!(event.customer_email, email);
        assert_eq!(event.event_name, "Sign Up");
        assert!(event.metadata.contains_key("source"));
    }

    #[test]
    fn test_sale_event_creation() {
        let customer_id = "test-customer".to_string();
        let amount = 2999; // $29.99
        
        let event = DubSaleEvent::new_subscription(customer_id.clone(), amount);
        
        assert_eq!(event.customer_id, customer_id);
        assert_eq!(event.amount_cents, amount);
        assert_eq!(event.currency, "USD");
        assert_eq!(event.event_name, "Subscription");
        assert!(event.metadata.contains_key("product_type"));
    }

    #[test]
    fn test_event_metadata() {
        let user_id = Uuid::new_v4();
        let event = DubLeadEvent::new_signup(user_id, "test@example.com".to_string())
            .with_metadata("plan".to_string(), serde_json::Value::String("pro".to_string()));
        
        assert!(event.metadata.contains_key("plan"));
        assert_eq!(event.metadata["plan"], serde_json::Value::String("pro".to_string()));
    }
}
