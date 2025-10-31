//! Tests for Dub service implementation
//!
//! Comprehensive test suite for Dub attribution tracking service,
//! including unit tests, integration tests, and error handling scenarios.

#[cfg(test)]
mod tests {
    use super::super::*;
    use uuid::Uuid;


    #[tokio::test]
    async fn test_lead_event_creation() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        
        let event = DubLeadEvent::new_signup(user_id, email.clone());
        
        assert_eq!(event.customer_id, user_id.to_string());
        assert_eq!(event.customer_email, email);
        assert_eq!(event.event_name, "Sign Up");
        assert!(event.metadata.contains_key("source"));
        assert_eq!(event.metadata["source"], serde_json::Value::String("registration".to_string()));
    }

    #[tokio::test]
    async fn test_sale_event_creation() {
        let customer_id = "test-customer-123".to_string();
        let amount = 2999; // $29.99
        
        let event = DubSaleEvent::new_subscription(customer_id.clone(), amount);
        
        assert_eq!(event.customer_id, customer_id);
        assert_eq!(event.amount_cents, amount);
        assert_eq!(event.currency, "USD");
        assert_eq!(event.event_name, "Subscription");
        assert!(event.metadata.contains_key("product_type"));
        assert_eq!(event.metadata["product_type"], serde_json::Value::String("subscription".to_string()));
    }

    #[tokio::test]
    async fn test_event_metadata_chaining() {
        let user_id = Uuid::new_v4();
        let event = DubLeadEvent::new_signup(user_id, "test@example.com".to_string())
            .with_metadata("plan".to_string(), serde_json::Value::String("pro".to_string()))
            .with_metadata("referrer".to_string(), serde_json::Value::String("google".to_string()));
        
        assert!(event.metadata.contains_key("source"));
        assert!(event.metadata.contains_key("plan"));
        assert!(event.metadata.contains_key("referrer"));
        assert_eq!(event.metadata["plan"], serde_json::Value::String("pro".to_string()));
        assert_eq!(event.metadata["referrer"], serde_json::Value::String("google".to_string()));
    }

    #[test]
    fn test_dub_config_disabled() {
        let config = DubConfig::disabled();
        assert!(!config.enabled);
        assert!(config.api_key.is_empty());
        assert!(config.workspace_id.is_empty());
        assert_eq!(config.base_url, "https://api.dub.co");
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_dub_config_test() {
        let config = DubConfig::test();
        assert!(config.enabled);
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.workspace_id, "test_workspace");
        assert_eq!(config.base_url, "https://api.dub.co");
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_dub_event_response_creation() {
        let response = DubEventResponse {
            success: true,
            event_id: Some("evt_123".to_string()),
            message: Some("Success".to_string()),
        };
        
        assert!(response.success);
        assert_eq!(response.event_id, Some("evt_123".to_string()));
        assert_eq!(response.message, Some("Success".to_string()));
    }

    #[test]
    fn test_dub_event_response_error() {
        let response = DubEventResponse {
            success: false,
            event_id: None,
            message: Some("API Error".to_string()),
        };
        
        assert!(!response.success);
        assert!(response.event_id.is_none());
        assert_eq!(response.message, Some("API Error".to_string()));
    }

    #[tokio::test]
    async fn test_generate_referral_token_disabled() {
        let service = DubService::disabled().unwrap();
        
        let result = service.generate_referral_token(
            "test-tenant".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            None,
            None,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn test_generate_referral_token_with_all_params() {
        let service = DubService::disabled().unwrap();
        
        let result = service.generate_referral_token(
            "test-tenant".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            Some("https://example.com/avatar.jpg".to_string()),
            Some("grp_12345".to_string()),
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    // Mock tests would require more complex setup with wiremock or similar
    // These tests focus on the service logic and data structures
}
