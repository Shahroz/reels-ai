//! New custom analytics event structure for database insertions.
//!
//! Represents the data required to create a new analytics event in the database.
//! Used by custom event tracking for inserting new business events.
//! Does not include auto-generated fields like id and timestamp.
//! Request context in request_details, event-specific data in custom_details.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct NewAnalyticsEvent {
    pub event_name: String,
    pub user_id: Option<uuid::Uuid>,
    /// Request context data including ip_address, user_agent, browser_info, etc.
    pub request_details: serde_json::Value,
    /// Custom event details stored as JSONB - contains domain-specific data for custom events (e.g., gennodes_server_response)
    pub custom_details: serde_json::Value,
    /// Session identifier kept as separate column for efficient querying
    pub session_id: Option<String>,
}

impl NewAnalyticsEvent {
    /// Create a new custom event for authenticated users
    /// Request context data goes in request_details JSONB to maintain analytics capabilities
    /// Event-specific data goes in custom_details JSONB
    pub fn custom_authenticated(
        event_name: String,
        user_id: uuid::Uuid,
        request_details: serde_json::Value,  // Request context (ip_address, user_agent, browser_info, etc.)
        custom_details: serde_json::Value,   // Custom event specific data (e.g., gennodes_server_response)
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_name,
            user_id: Some(user_id),
            request_details,
            custom_details,
            session_id,
        }
    }

    /// Create a new custom event for anonymous users
    /// Request context data goes in request_details JSONB to maintain analytics capabilities
    /// Event-specific data goes in custom_details JSONB
    pub fn custom_anonymous(
        event_name: String,
        request_details: serde_json::Value,  // Request context (ip_address, user_agent, browser_info, etc.)
        custom_details: serde_json::Value,   // Custom event specific data
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_name,
            user_id: None,
            request_details,
            custom_details,
            session_id,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new_analytics_event_custom_authenticated() {
        let event = super::NewAnalyticsEvent::custom_authenticated(
            String::from("vocal_tour_generated"),
            uuid::Uuid::new_v4(),
            serde_json::json!({
                "ip_address": "127.0.0.1",
                "user_agent": "test_agent",
                "browser_info": {"family": "Chrome", "version": "91.0"}
            }),
            serde_json::json!({
                "gennodes_server_response": {"status": "success", "processing_time": 5000}
            }),
            Some(String::from("session_123")),
        );
        
        assert_eq!(event.event_name, "vocal_tour_generated");
        assert!(event.user_id.is_some());
        assert_eq!(event.request_details["ip_address"], "127.0.0.1");
        assert_eq!(event.custom_details["gennodes_server_response"]["status"], "success");
    }

    #[test]
    fn test_new_analytics_event_custom_anonymous() {
        let event = super::NewAnalyticsEvent::custom_anonymous(
            String::from("user_registration"),
            serde_json::json!({
                "ip_address": "192.168.1.100",
                "user_agent": "anonymous_agent"
            }),
            serde_json::json!({
                "registration_method": "email",
                "email_domain": "gmail.com"
            }),
            Some(String::from("session_456")),
        );
        
        assert_eq!(event.event_name, "user_registration");
        assert!(event.user_id.is_none());
        assert_eq!(event.request_details["ip_address"], "192.168.1.100");
        assert_eq!(event.custom_details["registration_method"], "email");
    }
} 