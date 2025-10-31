//! Custom analytics event database model for business logic tracking.
//!
//! Represents the core analytics event structure stored in the database.
//! Contains comprehensive metadata for cohort-based funnel analysis.
//! Supports only custom business events (no automated middleware tracking).
//! Request context in request_details JSONB, event-specific data in custom_details JSONB.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AnalyticsEvent {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(example = "vocal_tour_generated")]
    pub event_name: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub user_id: Option<uuid::Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request context data stored as JSONB - contains ip_address, user_agent, browser_info, etc.
    pub request_details: serde_json::Value,
    /// Custom event details stored as JSONB - contains domain-specific data for custom events (e.g., gennodes_server_response)
    pub custom_details: serde_json::Value,
    /// Session identifier kept as separate column for efficient querying
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_analytics_event_creation_authenticated() {
        let event = super::AnalyticsEvent {
            id: uuid::Uuid::new_v4(),
            event_name: String::from("user_dashboard_view"),
            user_id: Some(uuid::Uuid::new_v4()), // Authenticated user
            timestamp: chrono::Utc::now(),
            request_details: serde_json::json!({
                "ip_address": "127.0.0.1",
                "user_agent": "test_agent",
                "user_registration_date": "2025-01-01"
            }),
            custom_details: serde_json::Value::Null, // NULL for custom events without details
            session_id: Some(String::from("session_123")),
        };
        assert_eq!(event.event_name, "user_dashboard_view");
        assert!(event.user_id.is_some());
        assert_eq!(event.request_details["ip_address"], "127.0.0.1");
    }

    #[test]
    fn test_analytics_event_creation_anonymous() {
        let event = super::AnalyticsEvent {
            id: uuid::Uuid::new_v4(),
            event_name: String::from("landing_page_view"),
            user_id: None, // Anonymous user before login
            timestamp: chrono::Utc::now(),
            request_details: serde_json::json!({
                "ip_address": "192.168.1.100",
                "user_agent": "anonymous_agent"
            }),
            custom_details: serde_json::Value::Null, // NULL for custom events without details
            session_id: Some(String::from("session_456")),
        };
        assert_eq!(event.event_name, "landing_page_view");
        assert!(event.user_id.is_none());
        assert_eq!(event.request_details["ip_address"], "192.168.1.100");
    }
} 